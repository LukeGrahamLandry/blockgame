#[cfg(not(target_arch = "wasm32"))]
mod run {
    use std::{env, fs};
    use ffi_test::reference_extern;
    use mlua::Lua;

    pub fn main() {
        reference_extern();
        let lua = unsafe { Lua::unsafe_new() };
        let mut args = env::args();
        let _ = args.next();  // exe name
        let path = args.next().unwrap();
        let src = fs::read_to_string(&path).unwrap_or_else(|e| panic!("Reading {} {:?} CWD: {:?}", path, e, env::current_dir()));
        lua.load(&*src).exec().unwrap_or_else(|e| {
            panic!("{}", e);
        });
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    run::main();
}
