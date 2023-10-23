
/// The data of one block in a chunk.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Tile(pub u16);

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
