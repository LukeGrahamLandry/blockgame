use full_moon::ast::Ast;
use full_moon::{Error, parse};

pub mod translate;
pub mod strip_types;
pub mod ast;
mod parse;

pub fn to_ast(lua_src: &str) -> Result<Ast, Box<Error>> {
    Ok(parse(lua_src)?)
}

// test rust calling in to lua
#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;
    use full_moon::print;
    use crate::strip_types::StripTypes;
    use crate::translate::tojs;
    use full_moon::visitors::{VisitorMut};

    fn compare(lua_src: &str, name: &str) {
        let ast = full_moon::parse(lua_src).unwrap();
        let js_src = format!("\n function lua_main(wasm){{\n {} }}", tojs(ast.clone()));
        let lua_ast = StripTypes().visit_ast(ast);
        let lua_out = run_lua(&print(&lua_ast), name);
        let js_out = run_js(&js_src, name);
        println!("Compare: {}", name);
        assert_eq!(lua_out, js_out);
    }

    fn run_lua(src: &str, name: &str) -> String {
        let path = format!("target/{}.lua", name);
        fs::write(&path, src).unwrap();
        let output = Command::new("../target/debug/ffi_test").arg(&*path).output().unwrap();
        assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    fn run_js(src: &str, name: &str) -> String {
        let path = format!("target/{}.js", name);
        fs::write(&path, format!("{}\n{}\n{}", include_str!("runtime.js"), include_str!("../tests/ffi_test/wasm_setup.js"), src)).unwrap();
        let output = Command::new("node").arg(&*path).output().unwrap();
        assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    #[test]
    fn file_output_tests() {
        Command::new("cargo").args(["build", "--lib", "--target=wasm32-unknown-unknown"]).current_dir("tests/ffi_test").status().unwrap();
        Command::new("cargo").args(["build"]).current_dir("tests/ffi_test").status().unwrap();
        for file in fs::read_dir("tests").unwrap() {
            let file = file.unwrap();
            if file.file_type().unwrap().is_file() {
                let name = file.file_name().to_string_lossy().to_string();
                let content = fs::read_to_string(file.path()).unwrap();
                compare(&content, &name);
            }
        }
    }

    // TODO: make this redundant by adding test cases
    #[test]
    fn dont_crash_parsing_logic() {
        let src = include_str!("../../logic/world.lua");
        let js = tojs(full_moon::parse(src).unwrap());
    }
}
