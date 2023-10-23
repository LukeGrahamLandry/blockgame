use std::ffi::c_void;
use std::hint::black_box;
use std::mem;
use mlua::{Function, LightUserData};
use blockgame::{gen, State};
use blockgame::window::{App, WindowContext};
use mlua::prelude::*;
use blockgame::lua_api::{generate_chunk, reference_extern, update_mesh};

use common::pos::Tile;

fn main() {
    reference_extern();
    println!("file:///{}", concat!(env!("OUT_DIR"), "/gen.rs"));
    env_logger::init();
    pollster::block_on(WindowContext::run(State::new));
}

#[repr(C)]
struct LogicChunk {
    tiles: [Tile; 16*16*16],
}
