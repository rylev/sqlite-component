export CC_wasm32_wasi="$WASI_SDK_PATH"/bin/clang
export RUSTFLAGS="-Clink-arg=-L$WASI_SDK_PATH/lib/clang/18/lib/wasi/ -Clink-arg=-lclang_rt.builtins-wasm32"
cargo component build
