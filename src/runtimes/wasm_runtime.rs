use std::{
    fmt::Debug,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};

use wasmer::{wasmparser::Operator, BaseTunables, Engine, NativeEngineExt, Pages};
use wasmer_wasix::virtual_fs::TmpFileSystem;

use crate::{
    common::runtime::{InputData, LimitingTunables},
    compilers::CompiledCode,
};

use super::{CodeRuntime, ExecutionResult};

/// Runtime for wasm code.
/// This uses `wasmer` to run the code.
#[derive(Debug, Clone, Default)]
pub struct WasmRuntime;

/// Configuration for wasm runtime.
#[derive(Clone)]
pub struct WasmConfig {
    /// Amount of gas to be used by the code. <br/>
    /// Default: 0 (no limit) <br/>
    /// This is better than setting a time limit because it doesn't depend on the machine.
    pub gas: usize,

    /// Maximum amount of memory that can be used by the code. <br/>
    /// Default: 0 (no limit)
    /// Unit for this is pages, where each page is 64KiB.
    pub memory_limit: usize,

    /// Custom metering cost function.
    /// This is used to calculate the cost of each instruction.
    /// Default cost function: `|_| -> u64 { 1 }`
    #[allow(clippy::type_complexity)]
    pub cost_function: Option<Arc<dyn Fn(&Operator) -> u64 + Send + Sync>>,

    /// File containing stdin to be used by the code.
    pub stdin: InputData,

    /// Compiler that should be used to compile the code.
    /// Default: `WasmCompiler::Cranelift`
    pub compiler: WasmCompiler,
}

/// Sets the compiler that should be used to compile the code.
#[derive(Debug, Clone)]
pub enum WasmCompiler {
    /// Cranelift compiler. <br/>
    /// This is the default compiler. It compiles the code faster than LLVM, but the code runs slower.
    Cranelift,
    /// LLVM compiler. <br/>
    /// Has longer compile times than Cranelift, but produced bytecode is faster and more optimized. <br/>
    /// Requires `llvm` to be installed on the system.
    #[cfg(feature = "wasm-llvm")]
    LLVM,
}

impl WasmCompiler {
    /// Returns the compiler that should be used to compile the code.
    pub fn get_compiler(&self) -> impl wasmer::CompilerConfig {
        match self {
            Self::Cranelift => wasmer::Cranelift::default(),
            #[cfg(feature = "wasm-llvm")]
            Self::LLVM => wasmer_compiler_llvm::LLVM::default(),
        }
    }
}

impl Default for WasmCompiler {
    fn default() -> Self {
        Self::Cranelift
    }
}

impl Debug for WasmConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmConfig")
            .field("gas", &self.gas)
            .field("cost_function", &self.cost_function.is_some())
            .field("stdin", &self.stdin)
            .finish()
    }
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            gas: 0,
            memory_limit: 0,
            cost_function: None,
            stdin: InputData::Ignore,
            compiler: WasmCompiler::default(),
        }
    }
}

/// Additional data for wasm runtime.
/// This can be used by the compiler to pass additional data to the runtime.
#[derive(Debug, Clone, Default)]
pub struct WasmAdditionalData {
    /// Additional arguments to be passed to the code.
    pub args: Vec<String>,

    /// Files that should be mounted in the code.
    /// This will be mounted as `/sandbox` in the code.
    pub preopen_dir: Option<PathBuf>,
}

/// Wasm runtime error.
macro_rules! impl_wasm_error {
    ($($errn:ident $(=> $ft:ty)?),*) => {
        /// Wasm runtime error.
        /// This contains all possible errors that can occur while running the code.
        #[derive(Debug)]
        pub enum WasmRuntimeError {
            $(
                $errn $(($ft))?,
            )*
        }

        $(
            $(
                impl From<$ft> for WasmRuntimeError {
                    fn from(err: $ft) -> Self {
                        Self::$errn(err)
                    }
                }
            )?
        )*
    };
}

// Implementation of all errors.
impl_wasm_error!(
    IOCompileError => wasmer::IoCompileError,
    IOError => std::io::Error,
    WasiRuntimeError => wasmer_wasix::WasiRuntimeError,
    WasiError => wasmer_wasix::WasiError,
    InstantiationError => wasmer::InstantiationError,
    ExportError => wasmer::ExportError,
    RuntimeError => wasmer::RuntimeError,
    WasiStateCreationError => wasmer_wasix::WasiStateCreationError,
    FsError => wasmer_wasix::FsError
);

/// Runtime for wasm code.
impl CodeRuntime for WasmRuntime {
    /// Configuration for the runtime.
    type Config = WasmConfig;
    /// Additional compilation data.
    type AdditionalData = WasmAdditionalData;
    /// Error type for the runtime.
    type Error = WasmRuntimeError;

    /// Uses `wasmtime` to run the code.
    fn run(
        &self,
        code: &CompiledCode<Self>,
        config: Self::Config,
    ) -> Result<ExecutionResult, Self::Error> {
        // Create engine with metering.
        let compiler_config = if config.gas != 0 {
            // Get cost function.
            let cost_function = config
                .cost_function
                .unwrap_or_else(|| Arc::new(|_| -> u64 { 1 }));
            // Wrap cost function.
            let cost_function = move |op: &Operator| -> u64 { cost_function(op) };
            // Create metering middleware.
            let metering = Arc::new(wasmer_middlewares::Metering::new(
                config.gas as u64,
                cost_function,
            ));

            let mut compiler_config = config.compiler.get_compiler();
            wasmer::CompilerConfig::push_middleware(&mut compiler_config, metering);
            compiler_config
        } else {
            config.compiler.get_compiler()
        };

        // Create engine
        let mut engine: Engine = wasmer::EngineBuilder::new(compiler_config).into();

        // Set memory limit.
        if config.memory_limit != 0 {
            let base = BaseTunables::for_target(&wasmer::Target::default());
            let memory_limit_tunables =
                LimitingTunables::new(Pages(config.memory_limit as u32), base);
            engine.set_tunables(memory_limit_tunables);
        }

        // Create store.
        let mut store = wasmer::Store::new(engine);
        let module = wasmer::Module::from_file(&store, code.executable.as_ref().unwrap())?;

        // Crate wasi pipes.
        let (mut stdin_tx, stdin_rx) = wasmer_wasix::Pipe::channel();
        let (stdout_tx, mut stdout_rx) = wasmer_wasix::Pipe::channel();
        let (stderr_tx, mut stderr_rx) = wasmer_wasix::Pipe::channel();

        // Write stdin to pipe.
        match &config.stdin {
            InputData::String(input) => {
                stdin_tx.write_all(input.as_bytes())?;
                stdin_tx.write_all(b"\n")?; // Add a newline to the end of input.
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
            .args(&code.additional_data.args);

        // Add preopen dir if present.
        if let Some(dir) = &code.additional_data.preopen_dir {
            // Get host fs.
            let host_fs: Arc<dyn wasmer_wasix::virtual_fs::FileSystem + Send + Sync + 'static> =
                Arc::new(wasmer_wasix::virtual_fs::host_fs::FileSystem::default());

            // Create tmp fs and mount host fs.
            let tmp_fs = TmpFileSystem::new();
            tmp_fs.mount("/sandbox".into(), &host_fs, dir.clone())?;
            wasi_env = wasi_env.sandbox_fs(tmp_fs);
        }

        let mut wasi_env = wasi_env.finalize(&mut store)?;

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

    #[test]
    fn wasm_test_security() {
        // Try to create file (should panic)
        let code = r#"
            fn main() {
                std::fs::File::create("test.txt").unwrap();
            }
        "#;

        let compiled_code = RustCompiler
            .compile(&mut code.as_bytes(), Default::default())
            .unwrap();
        let result = WasmRuntime.run(&compiled_code, Default::default());

        assert!(result.is_err());
    }

    #[test]
    fn wasm_test_gas_cost_ok() {
        let code = r#"
            fn main() {
                println!("Hello, world!");
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
                    gas: 5000,
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(result.stdout, Some("Hello, world!\n".to_owned()));
    }

    #[test]
    #[should_panic]
    fn wasm_test_gas_cost_exceeded() {
        let code = r#"
            fn main() {
                for _ in 0..1000000 {
                    println!("Hello, world!")
                }
            }
        "#;

        let compiled_code = RustCompiler
            .compile(&mut code.as_bytes(), Default::default())
            .unwrap();

        let _result = WasmRuntime
            .run(
                &compiled_code,
                WasmConfig {
                    gas: 100,
                    ..Default::default()
                },
            )
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn wasm_test_memory_limit_exceeded() {
        let code = r#"
            fn main() {
                let mut v = Vec::new();
                for _ in 0..10000000 {
                    v.push(0);
                }
            }
        "#;

        let compiled_code = RustCompiler
            .compile(&mut code.as_bytes(), Default::default())
            .unwrap();

        let _result = WasmRuntime
            .run(
                &compiled_code,
                WasmConfig {
                    memory_limit: 100,
                    ..Default::default()
                },
            )
            .unwrap();
    }
}
