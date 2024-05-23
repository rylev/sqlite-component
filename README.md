# SQLite Component

A component that embeds a SQLite engine inside of it.

## Building 

In order to build this component you need:
* The [wasi-sdk](https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-22)
* The env variable `WASI_SDK_PATH` pointing to the location of the wasi-sdk on your machine.
* [cargo-component](https://github.com/bytecodealliance/cargo-component)

Then run `./build.sh`.