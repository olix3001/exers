# Exers :computer:

Exers is a rust library for compiling and running code in different languages and runtimes.

## Usage example

```rust
fn main() {
    // Imports...

    let code = r#"
    fn main() {
        println!("Hello World!");
    }
    "#;

    let compiled_code = RustCompiler.compile(&mut code.as_bytes(), Default::default());
    let result = WasmRuntime.run(&compiled_code, Default::default()).unwrap();
}
```

## Supported languages :books:

| Language   | Supported Runtimes | Required Dependencies      |
| ---------- | ------------------ | -------------------------- |
| Rust       | Wasm, Native       | Rustc                      |
| C++        | Wasm, Native       | clang++                    |
| Python     | Native             | python3, Cython (optional) |
| JavaScript | None               | ---                        |
| C#         | None               | ---                        |
| Go         | None               | ---                        |

## Available runtimes :running_man:

| Runtime       | Status      |
| ------------- | ----------- |
| WASM          | Implemented |
| Native        | Implemented |
| Jailed        | Not working |
| Firecracker   | Not started |
| Docker/Podman | Not started |

## Contributing :handshake:

If you want to contribute to this project, please keep my code style and formatting. I use `rustfmt` to format my code. Please also make sure that your code compiles and that all tests pass. If you want to add a new language or runtime, remember to write tests and comment your code well.

Commits should follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification.

## Requirements :clipboard:

### WASM

If you want to use the WASM runtime, you need to install the `wasm32-wasi` target for rustc. You can do this by running `rustup target add wasm32-wasi`.

For C++ you need to install `wasi-sdk` or other WASI sdk/libc and specify
`WASI_SDK` environment variable to point to the sdk.

### Native

Native runtime just requires dependencies for the language you want to use.

## Additional features :sparkles:

### wasm-llvm

This feature allows you to use the LLVM backend for the WASM runtime.
LLVM offers better performance, but has longer compilation times.

### cython

This feature allows you to use Cython for the Python runtime.
This makes code execution faster, but requires Cython to be installed.

### Bundled :package: (planned)
contains all the dependencies for all the languages and runtimes, so you don't have to install them yourself. This may be useful for some use cases, but it will make the library much larger (probably over 1GB).

## Examples :page_facing_up:

Examples can be found in the `examples` directory. To run them, you need to install the required dependencies for the languages you want to use. You can then run the examples with `cargo run --example <example_name>`.

## Dockerfile :whale:

This project contains a Dockerfile that can be used to build a docker image with all the required dependencies for all the languages and runtimes. This image can be used to base your own images on :smile:.
I'm currently working on minimizing the size of the image (currently about 2GB) and allowing you to choose which languages and runtimes you want to include.

**Warning:** I've not tested the image yet, so it might not work for some languages and runtimes.
