use std::sync::{Arc, Mutex};

use crate::{
    common::compiler::check_program_installed,
    runtimes::{
        native_runtime::{NativeAdditionalData, NativeRuntime},
        wasm_runtime::WasmRuntime,
    },
};

use super::Compiler;

/// Javascript compiler.
/// This uses javy to compile the code to a wasm module. And runs the code in a nodejs environment for native modules.
/// Javy is bundled with this crate and will be downloaded and installed automatically.
pub struct JsCompiler;

impl Compiler<NativeRuntime> for JsCompiler {
    type Config = ();

    fn compile(
        &self,
        code: &mut impl std::io::Read,
        _config: Self::Config,
    ) -> crate::common::compiler::CompilationResult<super::CompiledCode<NativeRuntime>> {
        // Get temporary directory
        let temp_dir = tempfile::tempdir().unwrap();

        // Create code file in temporary directory
        let mut code_file = std::fs::File::create(temp_dir.path().join("code.js")).unwrap();

        // Copy code to code file
        std::io::copy(code, &mut code_file).unwrap();

        // Return compiled code that uses nodejs to run the code (first ensure that nodejs is installed)
        check_program_installed("node").unwrap();
        Ok(super::CompiledCode {
            executable: Some(temp_dir.path().join("code.js")),
            temp_dir_handle: Arc::new(Mutex::new(Some(temp_dir))),
            additional_data: NativeAdditionalData {
                program: Some("node".to_string()),
            },
            runtime_marker: std::marker::PhantomData,
        })
    }
}

impl Compiler<WasmRuntime> for JsCompiler {
    type Config = ();

    /// Compile javascript code to wasm using javy.
    ///
    /// **WARNING**: Output from console.log will be written to stderr instead of stdout. (This will be fixed in the future)
    ///
    /// ### How to print to stdout? (temporary workaround)
    /// ```js,ignore
    /// function writeStdout(data) {
    ///     const encoder = new TextEncoder();
    ///     const buffer = new Uint8Array(encoder.encode(JSON.stringify(data)));
    ///     const fd = 1; // stdout
    ///     Javy.IO.writeSync(fd, buffer);
    /// }
    /// ```
    #[allow(unused_variables, unreachable_code)]
    fn compile(
        &self,
        code: &mut impl std::io::Read,
        _config: Self::Config,
    ) -> crate::common::compiler::CompilationResult<super::CompiledCode<WasmRuntime>> {
        // Get temporary directory
        let temp_dir = tempfile::tempdir().unwrap();

        // Create code file in temporary directory
        let mut code_file = std::fs::File::create(temp_dir.path().join("code.js")).unwrap();

        // Copy code to code file
        std::io::copy(code, &mut code_file).unwrap();

        // Compile code to wasm using javy
        let javy_path = std::env::var("JAVY_PATH").expect("JAVY_PATH environment variable not set");
        std::process::Command::new(format!("{}/javy", javy_path))
            .args([
                "compile",
                "-o",
                temp_dir.path().join("code.wasm").to_str().unwrap(),
                temp_dir.path().join("code.js").to_str().unwrap(),
            ])
            .output()?;

        // Return compiled code for wasm runtime
        Ok(super::CompiledCode {
            executable: Some(temp_dir.path().join("code.wasm")),
            temp_dir_handle: Arc::new(Mutex::new(Some(temp_dir))),
            additional_data: Default::default(),
            runtime_marker: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::runtimes::CodeRuntime;

    use super::*;

    #[test]
    fn test_compile_native() {
        let mut code = std::io::Cursor::new("console.log('Hello World!');".as_bytes());
        let compiled_code = JsCompiler.compile(&mut code, Default::default()).unwrap();
        let result = NativeRuntime
            .run(&compiled_code, Default::default())
            .unwrap();

        assert_eq!(result.stdout, Some("Hello World!\n".to_string()));
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_compile_wasm() {
        let mut code = std::io::Cursor::new("console.log('Hello World!');".as_bytes());
        let compiled_code = JsCompiler.compile(&mut code, Default::default()).unwrap();
        let result = WasmRuntime.run(&compiled_code, Default::default()).unwrap();

        assert_eq!(result.stderr, Some("Hello World!\n".to_string()));
    }
}
