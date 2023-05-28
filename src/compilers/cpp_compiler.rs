use std::{
    io,
    sync::{Arc, Mutex},
};

use crate::{
    common::compiler::{check_program_installed, CompilationError, CompilationResult, OptLevel},
    runtimes::CodeRuntime,
};

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
    pub fn compile_with_args<R: CodeRuntime>(
        &self,
        code: &mut impl io::Read,
        command: &str,
        config: CppCompilerConfig,
        args: &[&str],
        output_name: &str,
    ) -> CompilationResult<CompiledCode<R>>
    where
        Self: Compiler<R>,
    {
        // Create temporary directory for code and executable.
        let temp_dir = tempfile::Builder::new().prefix("exerscpp-").tempdir()?;

        // Create temporary file for code.
        let mut code_file = tempfile::Builder::new()
            .prefix("code-")
            .suffix(".cpp")
            .tempfile_in(temp_dir.path())?;
        io::copy(code, &mut code_file)?;

        // Compile the code using `rustc` command with given arguments.
        let mut command = std::process::Command::new(command);
        command.stderr(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::null());
        command.stdin(std::process::Stdio::null());
        command.current_dir(temp_dir.path());
        command.args(args);
        command.arg(code_file.path());

        // Add compiler arguments.
        for arg in config.clone().into_args() {
            command.arg(arg);
        }

        command.arg("-o");
        command.arg(temp_dir.path().join(output_name));

        println!("{:?}", command);
        let output = command.spawn()?.wait_with_output()?;

        // Check if compilation was successful.
        if !output.status.success() {
            return Err(CompilationError::CompilationFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Return compiled code.
        Ok(CompiledCode {
            executable: Some(temp_dir.path().join(output_name)),
            temp_dir_handle: Arc::new(Mutex::new(Some(temp_dir))),
            additional_data: R::AdditionalData::default(),
            runtime_marker: std::marker::PhantomData,
        })
    }
}

/// Comfiguration for C++ compiler.
#[derive(Debug, Clone)]
pub struct CppCompilerConfig {
    /// Opt level for C++ compiler. <br/>
    /// This is passed to `clang++` command using `-O<level>` argument.
    pub opt_level: OptLevel,

    /// Additional flags for C++ compiler.
    pub additional_flags: Vec<String>,
}

impl CppCompilerConfig {
    /// Creates new fully optimized configuration.
    pub fn optimized() -> Self {
        Self {
            opt_level: OptLevel::O3,
            ..Default::default()
        }
    }
}

// Default configuration for C++ compiler.
impl Default for CppCompilerConfig {
    fn default() -> Self {
        Self {
            opt_level: OptLevel::None,
            additional_flags: Vec::new(),
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

        // Add additional flags.
        args.extend(self.additional_flags);

        args
    }
}

/// Compiler for wasm runtime.
#[cfg(feature = "wasm")]
use crate::runtimes::wasm_runtime::WasmRuntime;
#[cfg(feature = "wasm")]
impl Compiler<WasmRuntime> for CppCompiler {
    type Config = CppCompilerConfig;

    fn compile(
        &self,
        code: &mut impl io::Read,
        config: Self::Config,
    ) -> CompilationResult<CompiledCode<WasmRuntime>> {
        check_program_installed("clang++");
        let sysroot_path = std::env::var("WASI_SYSROOT").expect(
            "WASI_SYSROOT environment variable not set. Consider installing wasi-sdk or wasi-libc.",
        );

        self.compile_with_args(
            code,
            "clang++",
            config,
            &[
                "--target=wasm32-wasi",
                format!("--sysroot={}", sysroot_path).as_str(),
            ],
            "executable.wasm",
        )
    }
}

/// Compiler for native runtime.
#[cfg(feature = "native")]
use crate::runtimes::native_runtime::NativeRuntime;
#[cfg(feature = "native")]
impl Compiler<NativeRuntime> for CppCompiler {
    type Config = CppCompilerConfig;

    fn compile(
        &self,
        code: &mut impl io::Read,
        config: Self::Config,
    ) -> CompilationResult<CompiledCode<NativeRuntime>> {
        check_program_installed("clang++");
        self.compile_with_args(code, "clang++", config, &[], "executable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "native")]
    #[test]
    fn test_cpp_native_runtime() {
        let code = r#"
            #include <iostream>
            int main() {
                std::cout << "Hello, World!";
                return 0;
            }
        "#;

        let compiled_code = CppCompiler
            .compile(&mut code.as_bytes(), Default::default())
            .unwrap();
        let result = NativeRuntime
            .run(&compiled_code, Default::default())
            .unwrap();

        assert_eq!(result.stdout.unwrap(), "Hello, World!");
        assert_eq!(result.exit_code, 0);
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_cpp_compiler_wasm() {
        let code = r#"
            #include <iostream>
            int main() {
                std::cout << "Hello, World!";
                return 0;
            }
        "#;

        let compiled_code = CppCompiler
            .compile(&mut code.as_bytes(), Default::default())
            .unwrap();
        let result = WasmRuntime.run(&compiled_code, Default::default()).unwrap();

        assert_eq!(result.stdout.unwrap(), "Hello, World!");
        assert_eq!(result.stderr.unwrap(), "");
        assert_eq!(result.exit_code, 0);
    }
}
