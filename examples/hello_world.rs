// This example shows how to compile simple rust hello world program
// and run it using native runtime.
// This example requires only rustc to be installed.

use exers::{
    compilers::{rust_compiler::RustCompiler, Compiler},
    runtimes::{native_runtime::NativeRuntime, CodeRuntime},
};

fn main() {
    // First, let's write some code that we want to compile.
    // Here we will use rust hello world program.
    let code = r#"
        fn main() {
            println!("Hello world!");
        }
    "#;

    // Now, let's create compiler. We will use rust compiler here.
    // Rust compiler does not need any preconfiguration,
    // so we don't need to use ::new() method.
    let compiler = RustCompiler;

    // Now, let's compile our code. We will use default config.
    // Default config for rust compiler has opt level set to 0
    // and codegen-units set to 1.
    // You could also use custom config or ::optimized() method.
    // Compiler::compile() takes anything that implements Read trait as input.
    // So we need to use &mut code.as_bytes() instead of just &code as an input.
    let compiled_code = compiler
        .compile(&mut code.as_bytes(), Default::default())
        .unwrap();

    // Now, let's create our runtime. We will use native runtime here.
    // Native runtime also does not need any preconfiguration, so
    // again we don't need to use ::new() method.
    let runtime = NativeRuntime;

    // Now, let's run our code. We will use default config.
    // Note that here type for compiled_code is CompiledCode<NativeRuntime>.
    // In this case, rust can infer the type, but sometimes you might need
    // to specify it manually.
    let execution_result = runtime.run(&compiled_code, Default::default()).unwrap();

    // Now, let's print the result!
    println!("{:?}", execution_result);
}
