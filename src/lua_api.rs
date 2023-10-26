use crate::pos::Chunk;
use crate::State;
use std::hint::black_box;

#[cfg(not(target_arch = "wasm32"))]
pub mod lua {
    use mlua::{Function, LightUserData, Lua};
    use std::ffi::c_void;
    use crate::State;

    pub struct GameLogic {
        lua: Lua
    }

    impl GameLogic {
        pub fn new() -> Self {;
            let lua = unsafe {
                Lua::unsafe_new()
            };

            lua.load(include_str!(concat!(env!("OUT_DIR"), "/compiled.lua"))).exec().unwrap();

            Self { lua }
        }
        pub fn run_tick(&self, state: &mut State) {
            let tick_chunk: Function = self.lua.globals().get("run_tick").unwrap();
            let _: () = tick_chunk.call(LightUserData(state as *const _ as *mut c_void)).unwrap_or_else(|e| {
                panic!("{}", e);
            });
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod lua {
    use std::ffi::c_void;
    use crate::State;
    use wasm_bindgen::prelude::*;

    pub struct GameLogic {}

    impl GameLogic {
        pub fn new() -> Self {
            Self {}
        }

        pub fn run_tick(&self, state: &mut State) {
            tick_chunk(state);
        }
    }

    #[wasm_bindgen]
    extern "C" {
        fn tick_chunk(state: *mut State);
    }
}

#[no_mangle]
pub extern "C" fn generate_chunk(state: &mut State, chunk: &mut Chunk) {
    *chunk = state.world.get_or_gen(chunk.pos).clone();
    chunk.dirty.set(true);
}

#[no_mangle]
pub extern "C" fn update_mesh(state: &mut State, chunk: &mut Chunk) {
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
    black_box(funcs);
}