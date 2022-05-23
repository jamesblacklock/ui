BUILD_WEB=true wasm-pack build --no-typescript --target=no-modules --features=web --no-default-features
cd ../..
cargo run --bin pkg_web -- simple ./examples/simple/pkg/simple_web_bg.wasm ./examples/simple/pkg/simple_web_bg.js