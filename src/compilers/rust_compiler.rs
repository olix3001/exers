use std::{io, sync::{Arc, Mutex}};

use crate::{runtimes::CodeRuntime, common::compiler::OptLevel};

use super::{Compiler, CompiledCode, IntoArgs};

/// Rust compiler.
/// Compiles code using `rustc` command. <br/>
/// For configuration options see [`RustCompilerConfig`].
#[derive(Debug, Clone)]
pub struct RustCompiler;

// Common elements for all compilers.
impl RustCompiler {
    /// Compile the given code (as stream of bytes) and return the executable (in temporary file).
    /// This function is used by `Compiler` trait.
    /// This also takes additional arguments for `rustc` command.
    pub fn compile_with_args<R: CodeRuntime> (
            &self,
            code: &mut impl io::Read,
            config: RustCompilerConfig,
            args: &[&str]
        ) -> io::Result<CompiledCode<R>> where Self: Compiler<R> {
        // Create temporary directory for code and executable.
        let temp_dir = tempfile::Builder::new().prefix("code-").tempdir()?;

        // Create temporary file for code.
        let mut code_file = tempfile::Builder::new().prefix("code-").suffix(".rs").tempfile_in(temp_dir.path())?;
        io::copy(code, &mut code_file)?;

        // Compile the code using `rustc` command with given arguments.
        let mut command = std::process::Command::new("rustc");
        command.current_dir(temp_dir.path());
        command.args(args);
        command.arg(code_file.path());
        
        // Add compiler arguments.
        for arg in config.clone().into_args() {
            command.arg(arg);
        }

        command.arg("-o");
        command.arg(temp_dir.path().join("executable.wasm"));

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
            codegen_units: 1
        }
    }
}

// Default configuration for rust compiler.
impl Default for RustCompilerConfig {
    fn default() -> Self {
        Self {
            opt_level: OptLevel::None,
            codegen_units: 1
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
            args.push(match self.opt_level {
                OptLevel::O1 => "opt-level=1",
                OptLevel::O2 => "opt-level=2",
                OptLevel::O3 => "opt-level=3",
                OptLevel::Speed => "opt-level=s",
                OptLevel::Size => "opt-level=z",
                OptLevel::Custom(_) => panic!("Custom opt level not supported for rust."),
                _ => unreachable!()
            }.to_string());
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

    fn compile(&self, code: &mut impl io::Read, config: RustCompilerConfig) -> io::Result<CompiledCode<WasmRuntime>> {
        // Compile the code using `rustc` command with given arguments.
        self.compile_with_args(code, config, &["--target", "wasm32-wasi"])
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

        let compiled_code: CompiledCode<WasmRuntime> = RustCompiler.compile(&mut code, config).unwrap();
        let executable = compiled_code.executable.as_ref().unwrap();

        assert!(executable.exists());
    }
}