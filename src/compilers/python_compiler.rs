use std::{
    fmt::Debug,
    fs::File,
    sync::{Arc, Mutex},
};

#[allow(unused_imports)]
use crate::{
    common::compiler::{CompilationError, CompilationResult},
    runtimes::native_runtime::{NativeAdditionalData, NativeRuntime},
};

/// Include python wasm file if wasm feature is enabled. <br/>
/// This is from https://github.com/vmware-labs/webassembly-language-runtimes/releases <br/>
// #[cfg(feature = "wasm")]
// const PYTHON_WASM: &[u8] = include_bytes!("../../assets/python.wasm");

#[cfg(feature = "cython")]
use crate::common::compiler::check_program_installed;

#[cfg(feature = "cython")]
use super::cpp_compiler::CppCompiler;
use super::{Compiler, IntoArgs};

/// Python compiler. <br/>
/// Actually, python is not compiled, but this is used to create a temporary file containing the code. <br/>
/// Alternatively, you can enable `cython` feature to compile python code to C code and then compile it using C compiler.
#[derive(Debug, Clone)]
pub struct PythonCompiler;

/// Configuration for Python compiler.
pub struct PythonCompilerConfig {
    /// Python version to use. <br/>
    /// Default is `python3`.
    pub python_version: String,

    /// Whether to use cython to compile the code. <br/>
    /// This option is only available if `cython` feature is enabled.
    #[cfg(feature = "cython")]
    pub use_cython: bool,

    /// Configuration for C++ compiler. <br/>
    /// This is only used if `use_cython` is true.
    #[cfg(feature = "cython")]
    pub cpp_config: super::cpp_compiler::CppCompilerConfig,
}

impl Debug for PythonCompilerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PythonCompilerConfig")
            .field("python_version", &self.python_version)
            .finish()
    }
}

impl Clone for PythonCompilerConfig {
    fn clone(&self) -> Self {
        Self {
            python_version: self.python_version.clone(),
            #[cfg(feature = "cython")]
            use_cython: self.use_cython,
            #[cfg(feature = "cython")]
            cpp_config: self.cpp_config.clone(),
        }
    }
}

impl Default for PythonCompilerConfig {
    fn default() -> Self {
        Self {
            python_version: "python3".to_string(),
            #[cfg(feature = "cython")]
            use_cython: false,
            #[cfg(feature = "cython")]
            cpp_config: super::cpp_compiler::CppCompilerConfig::default(),
        }
    }
}

impl PythonCompilerConfig {
    #[cfg(feature = "cython")]
    fn cython_default() -> Self {
        Self {
            python_version: "python3".to_string(),
            use_cython: true,
            cpp_config: super::cpp_compiler::CppCompilerConfig::default(),
        }
    }
}

impl IntoArgs for PythonCompilerConfig {
    /// Convert this configuration to arguments for `python` command.
    fn into_args(self) -> Vec<String> {
        #[allow(unused_mut)]
        let mut args: Vec<String> = Vec::new();

        #[cfg(feature = "cython")]
        {
            if self.use_cython {
                args.push("-m".to_string());
                args.push("cython".to_string());
            }
        }

        args
    }
}

/// Compiler for native runtime.
impl Compiler<NativeRuntime> for PythonCompiler {
    /// Configuration for python compiler.
    type Config = PythonCompilerConfig;

    #[allow(unused_variables)]
    fn compile(
        &self,
        code: &mut impl std::io::Read,
        config: Self::Config,
    ) -> CompilationResult<super::CompiledCode<NativeRuntime>> {
        // Create temporary directory.
        let temp_dir = tempfile::Builder::new().prefix("exers-").tempdir()?;

        // Create file with python code
        let mut code_file = File::create(temp_dir.path().join("code.py"))?;
        std::io::copy(code, &mut code_file)?;

        // If cython is enabled, compile the code to C and then compile it using C compiler.
        #[cfg(feature = "cython")]
        {
            if config.use_cython {
                check_program_installed("cython")?;
                let mut command = std::process::Command::new("cython");
                command.stderr(std::process::Stdio::piped());
                command.stdout(std::process::Stdio::null());
                command.stdin(std::process::Stdio::null());

                command.current_dir(temp_dir.path());
                command.arg("code.py");
                command.arg("-3"); // Python 3
                command.arg("--cplus"); // C++ instead of C
                command.arg("--embed"); // Embed python interpreter into the code
                command.arg("-o");
                command.arg("code.cpp");

                let output = command.spawn()?.wait_with_output()?;
                if !output.status.success() {
                    return Err(CompilationError::CompilationFailed(
                        String::from_utf8_lossy(&output.stderr).to_string(),
                    ));
                }

                // Compile the generated C++ code.
                let mut code_stream = File::open(temp_dir.path().join("code.cpp"))?;
                let compiled = CppCompiler.compile(&mut code_stream, config.cpp_config)?;

                // Return the compiled code.
                return Ok(compiled);
            }
        }

        // If cython is not enabled, just return the path to the python file.
        Ok(super::CompiledCode {
            executable: Some(temp_dir.path().join("code.py")),
            temp_dir_handle: Arc::new(Mutex::new(Some(temp_dir))),
            additional_data: NativeAdditionalData {
                program: Some(config.python_version),
            },
            runtime_marker: std::marker::PhantomData,
        })
    }
}

/// Python compiler for wasm runtime.
#[cfg(feature = "wasm")]
use crate::runtimes::wasm_runtime::{WasmAdditionalData, WasmRuntime};

#[cfg(feature = "wasm")]
impl Compiler<WasmRuntime> for PythonCompiler {
    /// Configuration for python compiler.
    type Config = PythonCompilerConfig;

    #[allow(unused_variables, unreachable_code)]
    fn compile(
        &self,
        code: &mut impl std::io::Read,
        config: Self::Config,
    ) -> CompilationResult<super::CompiledCode<WasmRuntime>> {
        panic!("Python compiler is not yet supported for wasm runtime. For more information, see https://github.com/wasmerio/wasmer/issues/3170");
        // If cython is enabled, return an error.
        #[cfg(feature = "cython")]
        if config.use_cython {
            return Err(CompilationError::FeatureNotSupported(
                "Cython is not supported for wasm runtime.".to_string(),
            ));
        }

        // Create temporary directory.
        let temp_dir = tempfile::Builder::new().prefix("exers-").tempdir()?;

        // Copy python.wasm to the temporary directory.
        let mut wasm_file = File::create(temp_dir.path().join("python.wasm"))?;
        // std::io::copy(&mut PYTHON_WASM.clone(), &mut wasm_file)?;

        // Create sandbox directory.
        std::fs::create_dir(temp_dir.path().join("sandbox"))?;

        // Create file with python code
        let mut code_file = File::create(temp_dir.path().join("sandbox").join("code.py"))?;
        std::io::copy(code, &mut code_file)?;

        // Return the compiled code.
        let sandbox_path = temp_dir.path().join("sandbox");
        Ok(super::CompiledCode {
            executable: Some(temp_dir.path().join("python.wasm")),
            temp_dir_handle: Arc::new(Mutex::new(Some(temp_dir))),
            additional_data: WasmAdditionalData {
                args: vec!["/sandbox/code.py".into()],
                preopen_dir: Some(sandbox_path),
            },
            runtime_marker: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        compilers::Compiler,
        runtimes::{native_runtime::NativeRuntime, CodeRuntime},
    };

    #[test]
    fn test_python_compile_native_python3() {
        let code = r#"
print("Hello, world!", end="")
"#;

        let compiled = super::PythonCompiler
            .compile(&mut code.as_bytes(), Default::default())
            .unwrap();

        let result = NativeRuntime.run(&compiled, Default::default()).unwrap();
        assert_eq!(result.stdout, Some("Hello, world!".to_string()));
    }

    #[cfg(feature = "cython")]
    #[test]
    fn test_python_compile_native_cython() {
        use crate::compilers::python_compiler::PythonCompilerConfig;

        let code = r#"
print("Hello, world!", end="")
"#;

        let compiled = super::PythonCompiler
            .compile(&mut code.as_bytes(), PythonCompilerConfig::cython_default())
            .unwrap();

        let result = NativeRuntime.run(&compiled, Default::default()).unwrap();
        assert_eq!(result.stdout, Some("Hello, world!".to_string()));
    }

    //     #[cfg(feature = "wasm")]
    //     #[test]
    //     fn test_python_compile_wasm() {
    //         let code = r#"
    // print("Hello, world!", end="")
    // "#;

    //         let compiled = super::PythonCompiler
    //             .compile(&mut code.as_bytes(), Default::default())
    //             .unwrap();

    //         let result = super::WasmRuntime.run(&compiled, Default::default()).unwrap();
    //         assert_eq!(result.stdout, Some("Hello, world!".to_string()));
    //     }
}
