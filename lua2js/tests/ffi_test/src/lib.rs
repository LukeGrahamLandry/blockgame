use std::alloc::{GlobalAlloc, Layout};
use std::ffi::c_void;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::hint::black_box;


#[no_mangle]
pub extern "C" fn add(left: usize, right: usize) -> usize {
    left + right
}


static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
static LIVE_POINTERS: AtomicIsize = AtomicIsize::new(0);

#[no_mangle]
pub unsafe extern "C" fn lua_alloc(bytes: u32) -> *mut u8 {
    #[cfg(not(target_arch = "wasm32"))]
    panic!("lua_alloc should only be called on wasm. luajit handles everything.");

    #[cfg(target_arch = "wasm32")]
    {
        // TODO: make sure I'm not breaking alignment rules
        let layout = Layout::array::<u8>((bytes + 4) as usize).unwrap();
        let ptr = ALLOC.alloc_zeroed(layout);
        let int_ptr = ptr as *mut u32;
        *int_ptr = bytes;  // TODO: could use this size to implement rudimentary runtime type checking in debug mode.
        LIVE_POINTERS.fetch_add(1, Ordering::SeqCst);
        ptr.add(4)
    }
}

#[no_mangle]
pub unsafe extern "C" fn lua_drop(ptr: *mut u8) {
    // On native, allocations are handled by luajit so this is a no-op
    #[cfg(not(target_arch = "wasm32"))]
    return;

    #[cfg(target_arch = "wasm32")]
    {
        let real_ptr = ptr.sub(4);
        let int_ptr = ptr as *mut u32;
        let bytes = *int_ptr;
        let layout = Layout::array::<u8>((bytes + 4) as usize).unwrap();
        ALLOC.dealloc(real_ptr, layout);
        LIVE_POINTERS.fetch_sub(1, Ordering::SeqCst);
    }
}

#[repr(C)]
pub struct Pos {
    x: i32,
    y: i32,
    z: i32,
}

#[no_mangle]
pub extern "C" fn set_y(pos: &mut Pos, y: i32) {
    pos.y = y;
}

#[no_mangle]
pub extern "C" fn get_y(pos: &mut Pos) -> i32 {
    pos.y
}


pub fn reference_extern() {
    let funcs: &[*const extern "C" fn()] = &[
        add as _,
        get_y as _,
        set_y as _,
        lua_drop as _,
        lua_alloc as _,
    ];
    black_box(funcs);
}
