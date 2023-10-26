use std::fs;
use lua2js::strip_types::strip_types;
use lua2js::to_ast;
use lua2js::translate::tojs;

// TODO: its not rerunning if an image added to assets
fn main() {
    println!("cargo:rerun-if-changed=common/src/blocks.rs");
    let lua_src =  common::blocks::gen(&std::env::var("OUT_DIR").unwrap());

    // TODO: super dumb to need to manually list files here.
    println!("cargo:rerun-if-changed=logic/src/world.lua");
    println!("cargo:rerun-if-changed=logic/src/blocks.lua");
    println!("cargo:rerun-if-changed=logic/src/entities.lua");
    compile_lua(&[
        &*lua_src,
        include_str!("logic/world.lua"),
        include_str!("logic/blocks.lua"),
        include_str!("logic/entities.lua"),
    ]);
}

fn compile_lua(files: &[&str]) {
    let mut native_lua = String::new();
    let mut js_src = String::new();
    for src in files {
        native_lua += &*strip_types(src).unwrap();
        js_src += &*tojs(to_ast(src).unwrap());
    }
    let path = std::env::var("OUT_DIR").unwrap() + "/compiled.lua";
    fs::write(path, native_lua).unwrap();
    let path = "./assets/generated/compiled.lua.js";
    fs::write(path, js_src).unwrap();
}
