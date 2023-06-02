// Sometimes you might want to repeatedly use the same compiler
// and runtime without reconfiguring them. In this case, you can
// use RuntimeBuilder to create a preconfigured runtime.
// This example shows how to use It to make your code more concise.

use exers::{
    common::{
        builder::RuntimeBuilder,
        preprocessor::{Preprocessor, PreprocessorResult},
    },
    compilers::rust_compiler::RustCompiler,
    runtimes::native_runtime::NativeRuntime,
};

// Simple preprocessor that removes all $ signs from the code (just as an example)
#[derive()]
pub struct RemoveDolarSignsPreprocessor;

impl Preprocessor for RemoveDolarSignsPreprocessor {
    fn preprocess(&self, code: &str) -> PreprocessorResult<String> {
        Ok(code.replace("$", ""))
    }
}

fn main() {
    // As always, let's write some code that we want to compile.
    // I'll actually write two different programs here to show
    // how easy it is to reuse the same compiler and runtime.
    // This code should be fixed by preprocessor before compilation.
    let code1 = r#"
        $fn main() {
            println!("Hi! I'm the first program!");
        }$
    "#;

    let code2 = r#"
        fn main() {
            print$ln!("And $I'm the second program!");
        }
    "#;

    // Now, let's create our new runtime.
    // We will use native runtime here.
    let rust_native_runtime = RuntimeBuilder::new()
        .preprocessor(RemoveDolarSignsPreprocessor)
        .compiler(RustCompiler, None) // Compiler config is not needed.
        .runtime(NativeRuntime, None) // Runtime config is not needed.
        .build()
        .unwrap(); // We need to unwrap here because builder can fail.

    // Now, as we have our runtime, let's run our two example programs!
    println!(
        "First program: {:?}",
        rust_native_runtime(&mut code1.as_bytes())
    );
    println!(
        "Second program: {:?}",
        rust_native_runtime(&mut code2.as_bytes())
    );
}
