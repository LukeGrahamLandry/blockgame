// TODO: its not rerunning if an image added to assets
fn main() {
    println!("cargo:rerun-if-changed=common/src/blocks.rs");
    common::blocks::gen(&std::env::var("OUT_DIR").unwrap());
}
