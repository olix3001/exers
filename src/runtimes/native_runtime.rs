use std::{io::Write, process::Stdio};

use crate::common::runtime::InputData;

use super::CodeRuntime;

/// Native runtime.
/// This runtime runs the code natively on the server.
/// This is the fastest runtime.
#[derive(Debug, Clone)]
pub struct NativeRuntime;

/// Configuration for native runtime.
#[derive(Debug, Clone)]
pub struct NativeConfig {
    /// File containing stdin to be used by the code.
    pub stdin: InputData,
}

impl Default for NativeConfig {
    fn default() -> Self {
        Self {
            stdin: InputData::Ignore,
        }
    }
}

/// Additional data for native runtime.
/// This is used to pass additional data from the compiler to the runtime.
#[derive(Debug, Clone)]
pub struct NativeAdditionalData {
    /// Program that should be used to run the code. <br/>
    /// Default is None, which means that the executable will be treated as a program.
    pub program: Option<String>,
}

impl Default for NativeAdditionalData {
    fn default() -> Self {
        Self { program: None }
    }
}

/// Runtime for native code.
impl CodeRuntime for NativeRuntime {
    /// Configuration for the runtime.
    type Config = NativeConfig;
    /// Additional compilation data.
    type AdditionalData = NativeAdditionalData;
    /// Error type for the runtime.
    type Error = std::io::Error;

    /// Runs the code natively on the server.
    fn run(
        &self,
        code: &crate::compilers::CompiledCode<Self>,
        config: Self::Config,
    ) -> Result<super::ExecutionResult, Self::Error> {
        // Create new process.
        let mut process = match &code.additional_data.program {
            Some(program) => {
                let mut cmd = std::process::Command::new(program);
                cmd.arg(&code.executable.as_ref().unwrap());
                cmd
            }
            None => std::process::Command::new(&code.executable.as_ref().unwrap()),
        };

        // Set stdin.
        match config.stdin {
            InputData::Ignore => {
                process.stdin(std::process::Stdio::null());
            }
            _ => {
                process.stdin(Stdio::piped());
            }
        };

        // Set stdout.
        process.stdout(Stdio::piped());
        // Set stderr.
        process.stderr(Stdio::piped());

        // Spawn the process.
        let mut process = process.spawn()?;

        // Start timer.
        let start_time = std::time::Instant::now();

        // Write to stdin.
        match config.stdin {
            InputData::Ignore => {}
            InputData::String(data) => {
                process.stdin.as_mut().unwrap().write_all(data.as_bytes())?;
            }
            InputData::File(path) => {
                let mut file = std::fs::File::open(path)?;
                std::io::copy(&mut file, process.stdin.as_mut().unwrap())?;
            }
        };

        // Wait for the process to finish.
        let output = process.wait_with_output()?;

        // Stop timer.
        let time_taken = start_time.elapsed();

        // Get stdout.
        let stdout = match output.stdout.len() {
            0 => None,
            _ => Some(String::from_utf8(output.stdout).unwrap()),
        };

        // Get stderr.
        let stderr = match output.stderr.len() {
            0 => None,
            _ => Some(String::from_utf8(output.stderr).unwrap()),
        };

        // Return the result.
        Ok(super::ExecutionResult {
            stdout,
            stderr,
            time_taken,
            exit_code: output.status.code().unwrap_or(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::compilers::{rust_compiler::RustCompiler, Compiler};

    use super::*;

    #[test]
    fn test_native_runtime() {
        let code = r#"
        fn main() {
            println!("Hello, world!");
        }
        "#;

        let compiled_code = RustCompiler
            .compile(&mut code.as_bytes(), Default::default())
            .unwrap();
        let result = NativeRuntime
            .run(&compiled_code, Default::default())
            .unwrap();

        assert_eq!(result.stdout, Some("Hello, world!\n".to_owned()));
    }
}
