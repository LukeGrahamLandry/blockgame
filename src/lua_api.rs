use std::cell::UnsafeCell;
use std::ops::Deref;
use mlua::{Lua, UserData, UserDataMethods, Value};
use mlua::prelude::LuaResult;
use common::pos::Tile;
use crate::pos::{BlockPos, Chunk, LocalPos};
use crate::{gen, State};
use crate::world::{LogicChunks, SharedObj};


// TODO: might need to be raw pointers

#[no_mangle]
pub extern "C" fn generate_chunk(state: &mut State, chunk: &mut Chunk) {
    println!("generate_chunk {:?}", chunk.pos);
    *chunk = state.world.get_or_gen(chunk.pos).clone();
    chunk.dirty.set(true);
    println!("after: {:?}", chunk.pos)
}

#[no_mangle]
pub extern "C" fn update_mesh(state: &mut State, chunk: &mut Chunk) {
    println!("update_mesh {:?}", chunk.pos);
    if chunk.dirty.get() {
        state.chunks.update_mesh(chunk.pos, chunk);
        chunk.dirty.set(false);
    }
}

pub fn reference_extern() {
    let funcs: &[*const extern "C" fn()] = &[
        generate_chunk as _,
        update_mesh as _,
    ];

}