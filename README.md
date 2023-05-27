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

| Language | Supported Runtimes | Required Dependencies |
| -------- | ------------------ | --------------------- |
| Rust     | Wasm               | Rustc                 |
| C++      | None               | ---                   |
| Python   | None               | ---                   |
| Java     | None               | ---                   |
| C#       | None               | ---                   |
| Go       | None               | ---                   |
| Ruby     | None               | ---                   |

## Available runtimes :running_man:

| Runtime     | Status                                       |
| ----------- | -------------------------------------------- |
| WASM        | In development, not ready for production use |
| Native      | Not started                                  |
| Jailed      | Not started                                  |
| Firecracker | Not started                                  |
