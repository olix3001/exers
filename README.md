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

| Language | Supported Runtimes | Required Dependencies      |
| -------- | ------------------ | -------------------------- |
| Rust     | Wasm, Native       | Rustc                      |
| C++      | Wasm, Native       | clang++                    |
| Python   | Native             | python3, Cython (optional) |
| Java     | None               | ---                        |
| C#       | None               | ---                        |
| Go       | None               | ---                        |
| Ruby     | None               | ---                        |

## Available runtimes :running_man:

| Runtime     | Status                      |
| ----------- | --------------------------- |
| WASM        | Implemented                 |
| Native      | Implemented                 |
| Jailed      | In development, not working |
| Firecracker | Not started                 |

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
