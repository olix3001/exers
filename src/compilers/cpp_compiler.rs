use std::{io, sync::{Arc, Mutex}};

use crate::{common::compiler::{OptLevel, check_program_installed}, runtimes::CodeRuntime};

use super::{CompiledCode, Compiler, IntoArgs};

/// C++ compiler.
/// Compiles code using `clang++` for native code and `em++` for wasm code.
/// For configuration options see [`CppCompilerConfig`].
#[derive(Debug, Clone)]
pub struct CppCompiler;

/// Common elements for all C++ compilers.
impl CppCompiler {
    /// Compile the given code (as stream of bytes) and return the executable (in temporary file).
    /// This function is used by `Compiler` trait.
    /// This also takes additional arguments for `clang++` command.
    pub fn compile_with_args<R: CodeRuntime> (
            &self,
            code: &mut impl io::Read,
            command: &str,
            config: CppCompilerConfig,
            args: &[&str]
        ) -> io::Result<CompiledCode<R>> where Self: Compiler<R> {
        // Create temporary directory for code and executable.
        let temp_dir = tempfile::Builder::new().prefix("exers-").tempdir()?;

        // Create temporary file for code.
        let mut code_file = tempfile::Builder::new().prefix("code-").suffix(".cpp").tempfile_in(temp_dir.path())?;
        io::copy(code, &mut code_file)?;

        // Compile the code using `rustc` command with given arguments.
        let mut command = std::process::Command::new(command);
        command.current_dir(temp_dir.path());
        command.args(args);
        command.arg(code_file.path());
        
        // Add compiler arguments.
        for arg in config.clone().into_args() {
            command.arg(arg);
        }

        command.arg("-o");
        command.arg(temp_dir.path().join("executable.wasm"));

        println!("{:?}", command);
        let output = command.spawn()?.wait_with_output()?;

        // Check if compilation was successful.
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Compilation failed."));
        }

        // Return compiled code.
        Ok(CompiledCode {
            executable: Some(temp_dir.path().join("executable.wasm")),
            temp_dir_handle: Arc::new(Mutex::new(Some(temp_dir))),
            additional_data: R::AdditionalData::default(),
            runtime_marker: std::marker::PhantomData
        })
    }
}

/// Comfiguration for C++ compiler.
#[derive(Debug, Clone)]
pub struct CppCompilerConfig {
    /// Opt level for C++ compiler. <br/>
    /// This is passed to `clang++` command using `-O<level>` argument.
    pub opt_level: OptLevel,
}

impl CppCompilerConfig {
    /// Creates new fully optimized configuration.
    pub fn optimized() -> Self {
        Self {
            opt_level: OptLevel::O3,
        }
    }
}

// Default configuration for C++ compiler.
impl Default for CppCompilerConfig {
    fn default() -> Self {
        Self {
            opt_level: OptLevel::None,
        }
    }
}

impl IntoArgs for CppCompilerConfig {
    fn into_args(self) -> Vec<String> {
        let mut args = Vec::new();

        // Add opt level.
        if !matches!(self.opt_level, OptLevel::None) {
            args.push(format!("-O{}", self.opt_level.as_stanard_opt_char()));
        }

        args
    }
}

/// Compiler for wasm runtime.
#[cfg(feature = "wasm")]
use crate::runtimes::wasm_runtime::WasmRuntime;
#[cfg(feature = "wasm")]
impl Compiler<WasmRuntime> for CppCompiler {
    type Config = CppCompilerConfig;

    fn compile(&self, code: &mut impl io::Read, config: Self::Config) -> io::Result<CompiledCode<WasmRuntime>> {
        check_program_installed("wasic++");
        self.compile_with_args(code, "wasic++", config, &[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpp_compiler_wasm() {
        let code =r#"
            #include <iostream>
            int main() {
                std::cout << "Hello, World!";
                return 0;
            }
        "#;

        let compiled_code = CppCompiler.compile(&mut code.as_bytes(), Default::default()).unwrap();
        let result = WasmRuntime::run(&compiled_code, Default::default()).unwrap();

        assert_eq!(result.stdout.unwrap(), "Hello, World!");
        assert_eq!(result.stderr.unwrap(), "");
        assert_eq!(result.exit_code, 0);
    }
}