// This example shows how to compile simple rust hello world program
// into webassembly and run it using wasi runtime.
// This example requires rustc and wasm32-wasi target to be installed.

// I recommend looking at examples/hello_world.rs first as it is
// commented in more detail.

use exers::{
    compilers::{rust_compiler::RustCompiler, Compiler},
    runtimes::{wasm_runtime::WasmRuntime, CodeRuntime},
};

fn main() {
    // First, let's write some code that we want to compile.
    // Here we will use rust hello world program.
    let code = r#"
        fn main() {
            println!("Hello world!");
        }
    "#;

    // Let's compile our code. We will use default config.
    let compiled_code = RustCompiler
        .compile(&mut code.as_bytes(), Default::default())
        .unwrap();

    // Now, let's create our runtime. We will use wasi runtime here.
    // It does not need any preconfiguration.
    let runtime = WasmRuntime;

    // Now, let's run our code.
    // I'll use default config here, but you can also use custom config.
    // Wasm supports many config options, so I recommend looking at them.
    let execution_result = runtime.run(&compiled_code, Default::default()).unwrap();

    // Now, let's print the result!
    println!("{:?}", execution_result);
}

// If you look closely at the output, you will see that time_taken is less
// than in native runtime. This is because wasm runtime only measures time
// to actually run the code, while native runtime has overhead of
// executing the host command.
