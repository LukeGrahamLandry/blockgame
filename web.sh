# cargo install wasm-bindgen-cli
# cargo install wasm-opt

mkdir "assets/generated"
cp "lua2js/src/runtime.js" "assets/generated/runtime.js"

if [ "$1" == "--release" ] ; then
  echo "Building for release..."
  RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo build --release --target=wasm32-unknown-unknown
  wasm-bindgen --out-dir assets/generated --web target/wasm32-unknown-unknown/release/blockgame.wasm
  echo "wasm-opt will probably take a thousand years..."
  wasm-opt -O assets/generated/blockgame_bg.wasm -o assets/generated/blockgame_bg.wasm
else
  echo "Building for debug..."
  RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo build --target=wasm32-unknown-unknown
  wasm-bindgen --out-dir assets/generated --web target/wasm32-unknown-unknown/debug/blockgame.wasm
fi
