use std::{fs::File, sync::{Arc, Mutex}};

use crate::{runtimes::native_runtime::{NativeRuntime, NativeAdditionalData}, common::compiler::check_program_installed};

use super::{IntoArgs, Compiler};
#[cfg(feature = "cython")]
use super::cpp_compiler::CppCompiler;

/// Python compiler. <br/>
/// Actually, python is not compiled, but this is used to create a temporary file containing the code. <br/>
/// Alternatively, you can enable `cython` feature to compile python code to C code and then compile it using C compiler.
#[derive(Debug, Clone)]
pub struct PythonCompiler;

/// Configuration for Python compiler.
#[derive(Debug, Clone)]
pub struct PythonCompilerConfig {
    /// Whether to use cython to compile the code. <br/>
    /// This option is only available if `cython` feature is enabled.
    #[cfg(feature = "cython")]
    pub use_cython: bool,

    /// Configuration for C++ compiler. <br/>
    /// This is only used if `use_cython` is true.
    #[cfg(feature = "cython")]
    pub cpp_config: super::cpp_compiler::CppCompilerConfig,
}

impl Default for PythonCompilerConfig {
    fn default() -> Self {
        Self {
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
            use_cython: true,
            cpp_config: super::cpp_compiler::CppCompilerConfig::default(),
        }
    }
}

impl IntoArgs for PythonCompilerConfig {
    /// Convert this configuration to arguments for `python` command.
    fn into_args(self) -> Vec<String> {
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

    fn compile(
        &self,
        code: &mut impl std::io::Read,
        config: Self::Config,
    ) -> std::io::Result<super::CompiledCode<NativeRuntime>> {
        // Create temporary directory.
        let temp_dir = tempfile::Builder::new()
            .prefix("exers-")
            .tempdir()?;

        // Create file with python code
        let mut code_file = File::create(temp_dir.path().join("code.py"))?;
        std::io::copy(code, &mut code_file)?;

        // If cython is enabled, compile the code to C and then compile it using C compiler.
        #[cfg(feature = "cython")]
        {
            if config.use_cython {
                check_program_installed("cython");
                let mut command = std::process::Command::new("cython");
                command.stderr(std::process::Stdio::null());
                command.stdout(std::process::Stdio::null());
                command.stdin(std::process::Stdio::null());

                command.current_dir(temp_dir.path());
                command.arg("code.py");
                command.arg("-3"); // Python 3
                command.arg("--cplus"); // C++ instead of C
                command.arg("-o");
                command.arg("code.cpp");

                command.spawn()?.wait_with_output()?;

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
                program: Some("python3".to_string())
            },
            runtime_marker: std::marker::PhantomData
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{compilers::Compiler, runtimes::{native_runtime::NativeRuntime, CodeRuntime}};

    #[test]
    fn test_python_compile_native_python3() {
        let code = r#"
print("Hello, world!", end="")
"#;

        let compiled = super::PythonCompiler.compile(
            &mut code.as_bytes(),
            Default::default()
        ).unwrap();

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

        let compiled = super::PythonCompiler.compile(
            &mut code.as_bytes(),
            PythonCompilerConfig::cython_default()
        ).unwrap();

        let result = NativeRuntime.run(&compiled, Default::default()).unwrap();
        assert_eq!(result.stdout, Some("Hello, world!".to_string()));
    }
}