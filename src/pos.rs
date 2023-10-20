use glam::Vec3;

pub const CHUNK_SIZE: usize = 16;

/// The data of one block in a chunk.
#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct Tile(pub u16);

/// The position of a block within a chunk. Default is the empty block.
#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct LocalPos(usize);

/// The absolute position of a block in the world. Logically (ChunkPos * CHUNK_SIZE)+LocalPos.
#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct BlockPos {
    x: usize,
    y: usize,
    z: usize
}

/// The position of a chunk in the world.
#[derive(Copy, Clone, Eq, PartialEq, Default, Hash)]
pub struct ChunkPos {
    pub x: usize,
    pub y: usize,
    pub z: usize
}

pub struct Chunk {
    tiles: Box<[Tile; Chunk::LENGTH]>
}

impl Chunk {
    const LENGTH: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

    pub fn empty() -> Self {
        Chunk {
            tiles: Box::new([Tile(0); Self::LENGTH]),
        }
    }

    pub fn get(&self, pos: LocalPos) -> Tile {
        self.tiles[pos.0]
    }

    pub fn set(&mut self, pos: LocalPos, block: Tile) {
        self.tiles[pos.0] = block;
    }
}

impl LocalPos {
    // TODO: Think about which ordering makes the most sense.
    //       Carefully do it the cache locality way based on this if I iterate over all blocks.
    pub fn new(x: usize, y: usize, z: usize) -> LocalPos {
        LocalPos((y * CHUNK_SIZE * CHUNK_SIZE) + (x * CHUNK_SIZE) + z)
    }

    // TODO: I like the idea of these fitting in a register but maybe its really dumb since now
    //       I have to do a bunch of work to actually use them.
    pub fn normalized(self) -> Vec3 {
        let y = self.0 / CHUNK_SIZE / CHUNK_SIZE;
        let x = (self.0 / CHUNK_SIZE) % CHUNK_SIZE;
        let z = self.0 % CHUNK_SIZE;
        Vec3::new(
            x as f32 / CHUNK_SIZE as f32,
            y as f32 / CHUNK_SIZE as f32,
            z as f32 / CHUNK_SIZE as f32,
        )
    }
}

impl ChunkPos {
    pub fn new(x: usize, y: usize, z: usize) -> ChunkPos {
        ChunkPos { x, y, z }
    }
}

impl BlockPos {
    pub fn of(chunk: ChunkPos, local: LocalPos) -> Self {
        todo!()
    }

    pub fn new(x: usize, y: usize, z: usize) -> Self {
        todo!()
    }

    pub fn chunk(&self) -> ChunkPos {
        todo!()
    }

    pub fn local(&self) -> LocalPos {
        todo!()
    }
}
