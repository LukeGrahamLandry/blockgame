use common::pos::Tile;
use crate::gen;
use crate::pos::Chunk;
use crate::worldgen::rand::random_numbers;

pub fn generate(chunk: &mut Chunk) {
    if chunk.pos.y < 0 {
        println!("generate {:?}", chunk.pos);
        let mut n = random_numbers((chunk.pos.x + chunk.pos.y + chunk.pos.z).abs() as u32);
        let block = (n.next().unwrap() as usize % gen::tiles::SOLID_COUNT) + 1;
        for pos in chunk.tiles.iter_mut() {
            *pos = Tile::new(block, true);
        }
    }
}

/// https://blog.orhun.dev/zero-deps-random-in-rust/
pub mod rand {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    pub fn random_seed() -> u64 {
        RandomState::new().build_hasher().finish()
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
