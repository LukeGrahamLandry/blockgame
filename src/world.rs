use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use crate::chunk_mesh::ChunkList;
use crate::gen;
use crate::pos::{Chunk, ChunkPos};

pub struct SharedObj<T>(pub(crate) Box<UnsafeCell<T>>);

pub struct LogicChunks {
    chunks: HashMap<ChunkPos, SharedObj<Chunk>>  // TODO: no double pointer
}

impl LogicChunks {
    pub fn new() -> Self {
        LogicChunks {
            chunks: Default::default(),
        }
    }

    pub fn update_meshes(&self, render: &mut ChunkList) {
        for (pos, chunk) in self.chunks.iter() {
            if chunk.dirty.get() {
                render.update_mesh(*pos, chunk);
                chunk.dirty.set(false);
            }
        }
    }

    pub fn get_or_gen(&mut self, pos: ChunkPos) -> &mut Chunk {
        if self.chunks.get_mut(&pos).is_some() {
            return self.chunks.get_mut(&pos).unwrap();  // rust fucking sucks apparently.
        }

        self.generate_chunk(pos);
        self.get_or_gen(pos)
    }

    pub fn generate_chunk(&mut self, pos: ChunkPos) {
        self.chunks.insert(pos, SharedObj(Box::new(UnsafeCell::new(Chunk::full(gen::tiles::empty, pos)))));
    }
}

impl<T> Deref for SharedObj<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.0.get()
        }
    }
}

impl<T> DerefMut for SharedObj<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.get_mut()
    }
}
