BUILD_WEB=true cargo build --lib --target=wasm32-unknown-unknown --release
cd ../..
mkdir -p ./examples/simple/dist
cargo run --bin pkg_web -- ./target/wasm32-unknown-unknown/release/simple_web.wasm ./examples/simple/dist/simple_web.js