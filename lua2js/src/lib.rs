#![allow(unused)]

use std::fs;
use std::process::Command;
use crate::translate::tojs;

mod translate;


fn log(src: &str) {
    let ast = full_moon::parse(src);
    println!("{}", tojs(&ast.unwrap()));
}

fn compare(lua_src: &str, name: &str) {
    let ast = full_moon::parse(lua_src).unwrap();
    let js_src = tojs(&ast);
    let lua_out = run_lua(lua_src, name);
    println!("lua says: {}", lua_out);
    let js_out = run_js(&js_src, name);
    println!("js says: {}", js_out);
    assert_eq!(lua_out, js_out);
}

fn run_lua(src: &str, name: &str) -> String {
    let path = format!("target/{}.lua", name);
    fs::write(&path, src).unwrap();
    let output = Command::new("luajit").arg(&*path).output().unwrap();
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn run_js(src: &str, name: &str) -> String {
    let path = format!("target/{}.js", name);
    fs::write(&path, src).unwrap();
    let output = Command::new("node").arg(&*path).output().unwrap();
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn file_output_tests() {
    for file in fs::read_dir("tests").unwrap() {
        let file = file.unwrap();
        let name = file.file_name().to_string_lossy().to_string();
        let content = fs::read_to_string(file.path()).unwrap();
        compare(&content, &name);
    }
}

#[test]
fn demo() {
    // compare("print((-1) % 10)", "demo");
    compare(include_str!("../tests/flow.lua"), "demo");
}