// The path here is a bit weird. This crate is in a workspace so the target folder is up a directory.
const wasmBuffer = require('fs').readFileSync("../target/wasm32-unknown-unknown/debug/ffi_test.wasm");
WebAssembly.instantiate(wasmBuffer).then(wasm => lua_main(wasm.instance.exports));
