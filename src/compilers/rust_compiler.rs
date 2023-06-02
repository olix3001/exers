use std::{
    io,
    sync::{Arc, Mutex},
};

use crate::{
    common::compiler::{check_program_installed, CompilationError, CompilationResult, OptLevel},
    runtimes::CodeRuntime,
};

use super::{CompiledCode, Compiler, IntoArgs};

/// Rust compiler.
/// Compiles code using `rustc` command. <br/>
/// For configuration options see [`RustCompilerConfig`].
#[derive(Debug, Clone)]
pub struct RustCompiler;

// Common elements for all rust compilers.
impl RustCompiler {
    /// Compile the given code (as stream of bytes) and return the executable (in temporary file).
    /// This function is used by `Compiler` trait.
    /// This also takes additional arguments for `rustc` command.
    pub fn compile_with_args<R: CodeRuntime>(
        &self,
        code: &mut impl io::Read,
        config: RustCompilerConfig,
        args: &[&str],
        output_name: &str,
    ) -> CompilationResult<CompiledCode<R>>
    where
        Self: Compiler<R>,
    {
        check_program_installed("rustc")?;

        // Create temporary directory for code and executable.
        let temp_dir = tempfile::Builder::new().prefix("exers-").tempdir()?;

        // Create temporary file for code.
        let mut code_file = tempfile::Builder::new()
            .prefix("code-")
            .suffix(".rs")
            .tempfile_in(temp_dir.path())?;
        io::copy(code, &mut code_file)?;

        // Compile the code using `rustc` command with given arguments.
        let mut command = std::process::Command::new("rustc");
        command.stderr(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::null());
        command.stdin(std::process::Stdio::null());
        command.current_dir(temp_dir.path());
        command.args(args);
        command.arg(code_file.path());

        // Add compiler arguments.
        for arg in config.into_args() {
            command.arg(arg);
        }

        command.arg("-o");
        command.arg(temp_dir.path().join(output_name));

        let output = command.spawn()?.wait_with_output()?;

        // Check if compilation was successful.
        if !output.status.success() {
            return Err(CompilationError::CompilationFailed(
                String::from_utf8_lossy(&output.stderr).into(),
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

/// Configuration for rust compiler.
#[derive(Debug, Clone)]
pub struct RustCompilerConfig {
    /// Opt level for rust compiler. <br/>
    /// This is passed to `rustc` command using `-C opt-level=<level>` argument.
    pub opt_level: OptLevel,
    /// Codegen units for rust compiler. <br/>
    /// This is passed to `rustc` command using `-C codegen-units=<units>` argument.
    pub codegen_units: u32,
}

impl RustCompilerConfig {
    /// Creates new fully optimized configuration.
    pub fn optimized() -> Self {
        Self {
            opt_level: OptLevel::O3,
            codegen_units: 1,
        }
    }
}

// Default configuration for rust compiler.
impl Default for RustCompilerConfig {
    fn default() -> Self {
        Self {
            opt_level: OptLevel::None,
            codegen_units: 1,
        }
    }
}

impl IntoArgs for RustCompilerConfig {
    /// Convert this configuration to arguments for `rustc` command.
    fn into_args(self) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();

        // Add opt level.
        if !matches!(self.opt_level, OptLevel::None) {
            args.push("-C".to_string());
            args.push(format!(
                "opt-level={}",
                self.opt_level.as_stanard_opt_char()
            ));
        }

        // Add codegen units.
        args.push("-C".to_string());
        args.push(format!("codegen-units={}", self.codegen_units));

        args
    }
}

/// Compiler for wasm runtime.
#[cfg(feature = "wasm")]
use crate::runtimes::wasm_runtime::WasmRuntime;
#[cfg(feature = "wasm")]
impl Compiler<WasmRuntime> for RustCompiler {
    type Config = RustCompilerConfig;

    fn compile(
        &self,
        code: &mut impl io::Read,
        config: RustCompilerConfig,
    ) -> CompilationResult<CompiledCode<WasmRuntime>> {
        // Compile the code using `rustc` command with given arguments.
        self.compile_with_args(
            code,
            config,
            &["--target", "wasm32-wasi"],
            "executable.wasm",
        )
    }
}

/// Compiler for native runtime.
#[cfg(feature = "native")]
use crate::runtimes::native_runtime::NativeRuntime;
#[cfg(feature = "native")]
impl Compiler<NativeRuntime> for RustCompiler {
    type Config = RustCompilerConfig;

    fn compile(
        &self,
        code: &mut impl io::Read,
        config: RustCompilerConfig,
    ) -> CompilationResult<CompiledCode<NativeRuntime>> {
        // Compile the code using `rustc` command with given arguments.
        self.compile_with_args(code, config, &[], "executable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "wasm")]
    fn test_compile_wasm() {
        let mut code = "fn main() { println!(\"Hello, world!\"); }".as_bytes();
        let config = RustCompilerConfig::default();

        let compiled_code: CompiledCode<WasmRuntime> =
            RustCompiler.compile(&mut code, config).unwrap();
        let executable = compiled_code.executable.as_ref().unwrap();

        assert!(executable.exists());
    }
}
