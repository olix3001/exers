use std::{
    fs::File,
    io::{Read, Write},
};

use crate::{common::runtime::InputData, compilers::CompiledCode};

use super::{CodeRuntime, ExecutionResult};

/// Runtime for wasm code.
/// This uses `wasmtime` to run the code.
#[derive(Debug, Clone, Default)]
pub struct WasmRuntime;

/// Configuration for wasm runtime.
#[derive(Debug, Clone)]
pub struct WasmConfig {
    /// Maximum run time in seconds. <br/>
    /// Default: 0 (no limit) <br/>
    /// **Note:** This is not implemented yet.
    pub max_run_time: usize,

    /// File containing stdin to be used by the code.
    pub stdin: InputData,
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            max_run_time: 0,
            stdin: InputData::Ignore,
        }
    }
}

/// Runtime for wasm code.
impl CodeRuntime for WasmRuntime {
    /// Configuration for the runtime.
    type Config = WasmConfig;
    /// Additional compilation data.
    type AdditionalData = ();
    /// Error type for the runtime.
    type Error = Box<dyn std::error::Error + Send + Sync>;

    /// Uses `wasmtime` to run the code.
    fn run(
        &self,
        code: &CompiledCode<Self>,
        config: Self::Config,
    ) -> Result<ExecutionResult, Self::Error> {
        // Create store.
        let mut store = wasmer::Store::default();
        let module = wasmer::Module::from_file(&store, &code.executable.as_ref().unwrap())?;

        // Crate wasi pipes.
        let (mut stdin_tx, stdin_rx) = wasmer_wasix::Pipe::channel();
        let (stdout_tx, mut stdout_rx) = wasmer_wasix::Pipe::channel();
        let (stderr_tx, mut stderr_rx) = wasmer_wasix::Pipe::channel();

        // Write stdin to pipe.
        match &config.stdin {
            InputData::String(input) => {
                stdin_tx.write_all(input.as_bytes())?;
                stdin_tx.write(b"\n")?; // Add a newline to the end of input.
            }
            InputData::File(path) => {
                let mut file = File::open(path)?;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                stdin_tx.write_all(&buf)?;
            }
            InputData::Ignore => {}
        }

        // Create wasi instance.
        let mut wasi_env = wasmer_wasix::WasiEnv::builder("wasi_program")
            .stdin(Box::new(stdin_rx))
            .stdout(Box::new(stdout_tx))
            .stderr(Box::new(stderr_tx))
            .finalize(&mut store)?;

        // Initialize wasi instance.
        let import_object = wasi_env.import_object(&mut store, &module)?;
        let instance = wasmer::Instance::new(&mut store, &module, &import_object)?;

        // Initialize wasi env.
        wasi_env.initialize(&mut store, instance.clone())?;

        // Get _start function.
        let start = instance.exports.get_function("_start")?;

        // Start time measurement.
        let start_time = std::time::Instant::now();

        // Run
        start.call(&mut store, &[])?;

        // End time measurement.
        let time_taken = start_time.elapsed();

        // Cleanup wasi env.
        wasi_env.cleanup(&mut store, None);

        // Get output from pipes.
        let mut stdout = String::new();
        let mut stderr = String::new();

        // Read pipes
        stdout_rx.read_to_string(&mut stdout)?;
        stderr_rx.read_to_string(&mut stderr)?;

        Ok(ExecutionResult {
            stdout: Some(stdout),
            stderr: Some(stderr),
            time_taken,
            exit_code: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::compilers::{rust_compiler::RustCompiler, Compiler};

    use super::*;

    #[test]
    fn test_wasm_runtime() {
        let code = r#"
            fn main() {
                println!("Hello, world!");
            }
        "#;

        let compiled_code = RustCompiler
            .compile(&mut code.as_bytes(), Default::default())
            .unwrap();
        let result = WasmRuntime.run(&compiled_code, Default::default()).unwrap();

        assert_eq!(result.stdout, Some("Hello, world!\n".to_owned()));
    }

    #[test]
    fn test_wasm_runtime_with_input() {
        let code = r#"
            fn main() {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                println!("Hello, {}!", input.trim());
            }
        "#;

        let compiled_code = RustCompiler
            .compile(&mut code.as_bytes(), Default::default())
            .unwrap();
        let result = WasmRuntime
            .run(
                &compiled_code,
                WasmConfig {
                    stdin: InputData::String("world".to_owned()),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(result.stdout, Some("Hello, world!\n".to_owned()));
    }

    #[test]
    fn test_wasm_time_measurement() {
        let code = r#"
            fn main() {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                println!("Hello, {}!", input.trim());
            }
        "#;

        let compiled_code = RustCompiler
            .compile(&mut code.as_bytes(), Default::default())
            .unwrap();
        let result = WasmRuntime
            .run(
                &compiled_code,
                WasmConfig {
                    stdin: InputData::String("world".to_owned()),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(result.stdout, Some("Hello, world!\n".to_owned()));
        assert!(result.time_taken.as_nanos() > 0);
    }
}
