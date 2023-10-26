use std::cell::Cell;
use glam::Vec3;
use common::pos::Tile;

pub const CHUNK_SIZE: usize = 16;

/// The position of a block within a chunk. Default is the empty block.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct LocalPos(usize);

// TODO: needs to be float?
/// The absolute position of a block in the world. Logically (ChunkPos * CHUNK_SIZE)+LocalPos.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32
}

// TODO: needs to be float?
/// The position of a chunk in the world.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32
}

/// Note: these are big! put them in a box
#[repr(C)]
#[derive(Clone)]
pub struct Chunk {
    pub(crate) pos: ChunkPos,
    pub tiles: [Tile; Chunk::LENGTH],
    pub dirty: Cell<bool>
}

impl Chunk {
    const LENGTH: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

    // TODO: dont return by value
    pub fn full(tile: Tile, pos: ChunkPos) -> Self {
        Chunk {
            pos,
            tiles: [tile; Self::LENGTH],
            dirty: Cell::new(true),
        }
    }

    pub fn get(&self, pos: LocalPos) -> Tile {
        self.tiles[pos.0]
    }

    pub fn set(&mut self, pos: LocalPos, block: Tile) {
        self.dirty.set(true);
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
    pub fn new(x: i32, y: i32, z: i32) -> ChunkPos {
        ChunkPos { x, y, z }
    }

    pub fn axis_distance(&self, other: &ChunkPos) -> u32 {
        (self.x.abs_diff(other.x).max(self.y.abs_diff(other.y)).max(self.z.abs_diff(other.z)))
    }
}

impl BlockPos {
    pub fn vec(pos: Vec3) -> BlockPos {
        BlockPos::new(pos.x as i32, pos.y as i32, pos.z as i32)
    }

    pub fn of(_chunk: ChunkPos, _local: LocalPos) -> Self {
        todo!()
    }

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        BlockPos { x, y, z }
    }

    pub fn chunk(&self) -> ChunkPos {
        ChunkPos::new((self.x / CHUNK_SIZE as i32), (self.y / CHUNK_SIZE as i32), (self.z / CHUNK_SIZE as i32))
    }

    pub fn local(&self) -> LocalPos {
        LocalPos::new((self.x.unsigned_abs() % CHUNK_SIZE as u32) as usize, (self.y.unsigned_abs() % CHUNK_SIZE as u32) as usize, (self.z.unsigned_abs() % CHUNK_SIZE as u32) as usize)
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
        const OFFSETS: [BlockPos; 6] = [
            BlockPos::new(0, 1, 0),
            BlockPos::new(0, -1, 0),
            BlockPos::new(1, 0, 0),
            BlockPos::new(-1, 0, 0),
            BlockPos::new(0, 0, 1),
            BlockPos::new(0, 0, -1),
        ];
        OFFSETS[self as usize]
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
