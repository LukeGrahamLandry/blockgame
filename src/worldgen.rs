use std::cell::UnsafeCell;
use std::collections::HashMap;
use common::pos::Tile;
use crate::chunk_mesh::ChunkList;
use crate::gen;
use crate::pos::{BlockPos, Chunk, ChunkPos};
use crate::worldgen::rand::{random_numbers, random_seed};

pub struct LogicChunks {
    pub(crate) chunks: HashMap<ChunkPos, Box<UnsafeCell<Chunk>>>,
}

impl LogicChunks {
    pub fn new() -> Self {
        LogicChunks {
            chunks: Default::default(),
        }
    }

    pub fn update_meshes(&self, render: &mut ChunkList) {
        for (pos, chunk) in self.chunks.iter() {
            let chunk = unsafe {&*chunk.get() };
            if chunk.dirty.get() {
                render.update_mesh(*pos, chunk);
                chunk.dirty.set(false);
            }
        }
    }

    pub fn get_or_gen(&mut self, pos: ChunkPos, render: &mut ChunkList) -> *mut Chunk {
        if let Some(chunk) = self.chunks.get(&pos) {
            return chunk.get();
        }

        let mut chunk = Chunk::full(gen::tiles::empty, pos);
        generate(&mut chunk);
        render.update_mesh(pos, &chunk);
        let chunk = Box::new(UnsafeCell::new(chunk));
        let ptr = chunk.get();
        self.chunks.insert(pos, chunk);

        ptr
    }

    pub fn get_rand(&mut self) -> *mut Chunk {
        let choice = random_numbers(random_seed()).next().unwrap() as usize % self.chunks.len();
        self.chunks.iter().nth(choice).unwrap().1.get()
    }

    pub fn gc(&mut self, player: BlockPos, render: &mut ChunkList) {
        let unload_radius = 10;
        let player = player.chunk();

        let mut count = 0;
        self.chunks.retain(|pos, _| {
            if player.axis_distance(pos) > unload_radius {
                count += 1;
                render.remove(*pos);
                false
            } else {
                true
            }
        });

        println!("gc cleared {} chunks", count);
    }
}

pub fn generate(chunk: &mut Chunk) {
    if chunk.pos.y < 0 {
        // println!("generate {:?}", chunk.pos);
        let mut n = random_numbers((chunk.pos.x + chunk.pos.y + chunk.pos.z).unsigned_abs());
        let block = (n.next().unwrap() as usize % gen::tiles::SOLID_COUNT) + 1;
        debug_assert!(!Tile::new(block, true).empty());
        for pos in chunk.tiles.iter_mut() {
            *pos = Tile::new(block, true);
        }
    }
}

/// https://blog.orhun.dev/zero-deps-random-in-rust/
pub mod rand {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    pub fn random_seed() -> u32 {
        (RandomState::new().build_hasher().finish() & u32::MAX as u64) as u32
    }

    // Pseudorandom number generator from the "Xorshift RNGs" paper by George Marsaglia.
    // https://github.com/rust-lang/rust/blob/1.55.0/library/core/src/slice/sort.rs#L559-L573
    pub fn random_numbers(seed: u32) -> impl Iterator<Item = u32> {
        let mut random = seed;
        std::iter::repeat_with(move || {
            random ^= random << 13;
            random ^= random >> 17;
            random ^= random << 5;
            random
        })
    }
}
