use crate::pos::Chunk;
use crate::State;
use std::hint::black_box;
use std::sync::atomic::{AtomicIsize, Ordering};
use common::pos::Tile;
use std::alloc::{GlobalAlloc, Layout};
use std::ptr;
use crate::worldgen::generate;

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
            let pos = state.camera.camera.pos;
            let _: () = tick_chunk.call((LightUserData(state as *const _ as *mut c_void), pos.x, pos.y, pos.z)).unwrap_or_else(|e| {
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
            let pos = state.camera.camera.pos;
            run_tick(state, pos.x, pos.y, pos.z);
        }
    }

    #[wasm_bindgen]
    extern "C" {
        fn run_tick(state: *mut State, playerx: f32, playery: f32, playerz: f32);
    }
}


#[cfg(target_arch = "wasm32")]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
static LIVE_POINTERS: AtomicIsize = AtomicIsize::new(0);

#[no_mangle]
pub unsafe extern "C" fn lua_alloc(bytes: u32) -> *mut u8 {
    #[cfg(not(target_arch = "wasm32"))]
    panic!("lua_alloc should only be called on wasm. luajit handles everything.");

    #[cfg(target_arch = "wasm32")]
    {
        debug_assert_eq!(bytes, 9000);
        debug_assert!(bytes > 1);  // 0 makes no sense and I'm using 1 for checking double free
        // TODO: make sure I'm not breaking alignment rules
        let layout = Layout::array::<u8>((bytes + 4) as usize).unwrap();
        let ptr = ALLOC.alloc_zeroed(layout);
        debug_assert_ne!(ptr, ptr::null_mut());
        let int_ptr = ptr as *mut u32;
        *int_ptr = bytes;  // TODO: could use this size to implement rudimentary runtime type checking in debug mode.
        LIVE_POINTERS.fetch_add(1, Ordering::SeqCst);
        ptr.add(4)
    }
}

#[no_mangle]
pub unsafe extern "C" fn lua_drop(ptr: *mut u8) {
    debug_assert_ne!(ptr, ptr::null_mut());
    // On native, allocations are handled by luajit so this is a no-op
    #[cfg(not(target_arch = "wasm32"))]
    return;

    #[cfg(target_arch = "wasm32")]
    {
        let real_ptr = ptr.sub(4);
        let int_ptr = real_ptr as *mut u32;
        let bytes = *int_ptr;

        #[cfg(debug_assertions)]
        {
            *int_ptr = 1;
        }
        debug_assert_ne!(bytes, 1, "double free");
        debug_assert_eq!(bytes, 9000);

        let layout = Layout::array::<u8>((bytes + 4) as usize).unwrap();
        ALLOC.dealloc(real_ptr, layout);
        LIVE_POINTERS.fetch_sub(1, Ordering::SeqCst);
    }
}

#[no_mangle]
pub extern "C" fn unload_chunk(state: &mut State, chunk: &mut Chunk) {
    state.chunks.remove(chunk.pos);
}

#[no_mangle]
pub extern "C" fn generate_chunk(state: &mut State, chunk: &mut Chunk, x: i32, y: i32, z: i32) {
    chunk.pos.x = x;
    chunk.pos.y = y;
    chunk.pos.z = z;
    // *chunk = state.world.get_or_gen(chunk.pos).clone();
    generate(chunk);
    chunk.dirty.set(true);
}

#[no_mangle]
pub extern "C" fn update_mesh(state: &mut State, chunk: &mut Chunk) {
    if chunk.dirty.get() {
        state.chunks.update_mesh(chunk.pos, chunk);
        chunk.dirty.set(false);
    }
}

// TODO: fix my lua transpiler so i can access fields and not write this stupid boilerplate.

#[no_mangle]
pub extern "C" fn chunk_set_block(chunk: &mut Chunk, index: u32, tile: u32) -> u32 {
    let old = chunk.tiles[index as usize];
    let new = Tile(tile as u16);
    chunk.tiles[index as usize] = new;
    if old != new {
        chunk.dirty.set(true);
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn chunk_get_block(chunk: &mut Chunk, index: u32) -> u32 {
    chunk.tiles[index as usize].0 as u32
}

pub fn reference_extern() {
    let funcs: &[*const extern "C" fn()] = &[
        generate_chunk as _,
        update_mesh as _,
        chunk_set_block as _,
        chunk_get_block as _,
        unload_chunk as _,
        lua_drop as _,
        lua_alloc as _,
    ];
    black_box(funcs);
}
