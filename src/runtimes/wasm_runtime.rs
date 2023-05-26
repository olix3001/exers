use std::{io::BufReader, path::PathBuf, fs::File, str::from_utf8};

use crate::compilers::CompiledCode;

use super::{CodeRuntime, ExecutionResult};
use wasmtime_wasi::WasiCtxBuilder;

#[derive(Debug, Clone, Default)]
pub struct WasmRuntime;

/// Configuration for wasm runtime.
#[derive(Debug, Clone)]
pub struct WasmConfig {
    /// Maximum memory size in bytes.
    /// Default: 256 MiB
    pub max_memory_size: usize,
    /// Maximum run time in seconds. <br/>
    /// Default: 0 (no limit) <br/>
    /// **Note:** This is not implemented yet.
    pub max_run_time: usize,

    /// File containing stdin to be used by the code.
    pub stdin: InputData,

    /// Custom wasm config.
    pub custom_config: wasmtime::Config
}

/// Represents input data for the code.
#[derive(Debug, Clone)]
pub enum InputData {
    /// Stdin will be read from the given file.
    File(PathBuf),
    /// Stdin will be read from the given string.
    String(String),
    /// Stdin will be ignored.
    Ignore
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            max_memory_size: 256 * 1024 * 1024,
            max_run_time: 0,
            stdin: InputData::Ignore,
            custom_config: wasmtime::Config::default()
        }
    }
}

/// Runtime for wasm code.
impl CodeRuntime for WasmRuntime {
    /// Configuration for the runtime.
    type Config = WasmConfig;
    /// Error type for the runtime.
    type Error = wasmtime::Error;

    /// Uses `wasmtime` to run the code.
    fn run(code: &CompiledCode<Self>, config: Self::Config) -> Result<ExecutionResult, Self::Error> {
        // Create config for wasmtime.
        let mut wasm_config = config.custom_config;

        // Set maximum memory size.
        wasm_config.static_memory_maximum_size(config.max_memory_size as u64);
        wasm_config.static_memory_forced(true);

        // Create wasi pipes.
        let stdout = wasi_common::pipe::WritePipe::new_in_memory();
        let stderr = wasi_common::pipe::WritePipe::new_in_memory();

        // Ensure everything is dropped before we try to read from the pipes.
        {
            // Create wasmtime engine.
            let engine = wasmtime::Engine::new(&wasm_config)?;

            // Read module from file.
            let module = wasmtime::Module::from_file(&engine, &code.executable.as_ref().unwrap())?;

            // Create wasmtime linker.
            let mut linker = wasmtime::Linker::new(&engine);
            wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

            // Create wasi context.
            let mut wasi = WasiCtxBuilder::new()
                .stdout(Box::new(stdout.clone()))
                .stderr(Box::new(stderr.clone()));

            // And more pipes.
            match config.stdin {
                InputData::File(path) => {
                    let file = File::open(path)?;
                    let reader = BufReader::new(file);
                    let stdin = wasi_common::pipe::ReadPipe::new(reader);
                    wasi = wasi.stdin(Box::new(stdin));
                },
                InputData::String(string) => {
                    let stdin = wasi_common::pipe::ReadPipe::from(string);
                    wasi = wasi.stdin(Box::new(stdin));
                },
                InputData::Ignore => {}
            }

            // Build wasi context.
            let wasi = wasi.build();

            // Create wasmtime store.
            let mut store = wasmtime::Store::new(&engine, wasi);

            // Link module.
            linker.module(&mut store, "", &module)?;

            // Get and run main function.
            linker
                .get_default(&mut store, "")?
                .typed::<(), ()>(&store)?
                .call(&mut store, ())?;

            // Explicitly drop store.
            drop(store);
        }

        // Parse stdout and stderr into strings.
        let stdout = match stdout.try_into_inner() {
            Ok(stdout) => {
                // Read stdout into string.
                from_utf8(stdout.into_inner().as_slice()).map(|s| s.to_owned()).ok()
            },
            Err(_) => None
        };

        let stderr = match stderr.try_into_inner() {
            Ok(stderr) => {
                // Read stderr into string.
                from_utf8(stderr.into_inner().as_slice()).map(|s| s.to_owned()).ok()
            },
            Err(_) => None
        };

        // Return execution result.
        Ok(ExecutionResult {
            stdout,
            stderr,
            exit_code: 0,
        })
    }
}   

#[cfg(test)]
mod tests {
    use crate::compilers::{rust::RustCompiler, Compiler};

    use super::*;

    #[test]
    fn test_wasm_runtime() {
        let code = r#"
            fn main() {
                println!("Hello, world!");
            }
        "#;

        let compiled_code = RustCompiler.compile(&mut code.as_bytes(), Default::default()).unwrap();
        let result = WasmRuntime::run(&compiled_code, Default::default()).unwrap();
    
        assert_eq!(result.stdout, Some("Hello, world!\n".to_owned()));
    }
}