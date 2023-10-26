use blockgame::State;
use blockgame::window::{App, WindowContext};
use blockgame::lua_api::reference_extern;
use common::pos::Tile;

fn main() {
    reference_extern();
    println!("file:///{}", concat!(env!("OUT_DIR"), "/gen.rs"));
    println!("file:///{}", concat!(env!("OUT_DIR"), "/compiled.lua"));
    env_logger::init();
    pollster::block_on(WindowContext::run(State::new));
}

#[repr(C)]
struct LogicChunk {
    tiles: [Tile; 16*16*16],
}
