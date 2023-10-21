use glam::Vec3;

pub const CHUNK_SIZE: usize = 16;

/// The data of one block in a chunk.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Tile(u16);

/// The position of a block within a chunk. Default is the empty block.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct LocalPos(usize);

// TODO: needs to be float?
/// The absolute position of a block in the world. Logically (ChunkPos * CHUNK_SIZE)+LocalPos.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct BlockPos {
    x: isize,
    y: isize,
    z: isize
}

// TODO: needs to be float?
/// The position of a chunk in the world.
#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct ChunkPos {
    pub x: isize,
    pub y: isize,
    pub z: isize
}

pub struct Chunk {
    tiles: Box<[Tile; Chunk::LENGTH]>,
    count: usize
}

impl Chunk {
    const LENGTH: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

    pub fn full(tile: Tile) -> Self {
        let count = if tile == Tile(0) { 0 } else { Self::LENGTH };
        Chunk {
            tiles: Box::new([tile; Self::LENGTH]),
            count,
        }
    }

    pub fn get(&self, pos: LocalPos) -> Tile {
        self.tiles[pos.0]
    }

    pub fn set(&mut self, pos: LocalPos, block: Tile) {
        let prev = self.tiles[pos.0];
        // TODO: it might be better to pay the cost of counting only when regenerating the mesh.
        if prev.empty() != block.empty() {
            if block.empty() {
                self.count -= 1;
            } else {
                self.count += 1;
            }
        }
        self.tiles[pos.0] = block;
    }

    // This allows the rendering to quickly know if it can just skip a chunk.
    // I suspect a lot of chunks will just be air and I want those to be cheap.
    pub fn is_empty(&self) -> bool {
        self.count == 0
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
    pub fn new(x: isize, y: isize, z: isize) -> ChunkPos {
        ChunkPos { x, y, z }
    }
}

impl BlockPos {
    pub fn of(chunk: ChunkPos, local: LocalPos) -> Self {
        todo!()
    }

    pub const fn new(x: isize, y: isize, z: isize) -> Self {
        BlockPos { x, y, z }
    }

    pub fn chunk(&self) -> ChunkPos {
        todo!()
    }

    pub fn local(&self) -> LocalPos {
        todo!()
    }
}

impl Tile {
    const SOLID: u16 = 1 << 15;
    pub const EMPTY: Tile = Self::new(0, false);

    pub const fn new(index: usize, is_solid: bool) -> Self {
        if is_solid {
            Tile(index as u16 | Self::SOLID)
        } else {
            Tile(index as u16)
        }
    }

    pub fn empty(self) -> bool {
        self.0 == 0
    }

    pub fn solid(self) -> bool {
        (self.0 & Self::SOLID) != 0
    }

    pub fn index(self) -> usize {
        (self.0 & !Self::SOLID) as usize
    }

    pub fn custom_render(&self) -> bool {
       !self.solid() && !self.empty()
    }
}

#[repr(u8)]
pub enum Direction {
    Up = 0,
    Down = 1,
    North = 2,
    South = 3,
    East = 4,
    West = 5,
}

// TODO: use this in chunk_mesh::add_cube?
#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct DirSet(u8);

impl Direction {
    pub fn offset(self) -> BlockPos {
        const offsets: [BlockPos; 6] = [
            BlockPos::new(0, 1, 0),
            BlockPos::new(0, -1, 0),
            BlockPos::new(1, 0, 0),
            BlockPos::new(-1, 0, 0),
            BlockPos::new(0, 0, 1),
            BlockPos::new(0, 0, -1),
        ];
        offsets[self as usize]
    }
}

impl DirSet {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn add(&mut self, dir: Direction) {
        self.0 |= 1 << dir as usize;
    }

    pub fn remove(&mut self, dir: Direction) {
        self.0 &= !(1 << dir as usize);
    }

    pub fn contains(&mut self, dir: Direction) -> bool{
        self.0 & (1 << dir as usize) != 0
    }
}


#[test]
fn tile_repr(){
    assert_eq!(Tile(0), Tile::EMPTY);
    assert_eq!(Tile::default(), Tile::EMPTY);
    assert!(Tile::EMPTY.empty());
}
