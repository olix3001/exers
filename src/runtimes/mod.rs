use crate::compilers::{CompiledCode, Compiler};

pub mod wasm_runtime;

/// Trait for every code runtime.
/// Represents a runtime that can be used to run some code.
pub trait CodeRuntime: Send + Sync + Sized {
    /// Run compiled code. Returns saved output (if any) and exit code.
    fn run<C: Compiler<Self>>(code: CompiledCode<C, Self>) -> ExecutionResult;
}

/// Configuration for runtimes.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Maximum time (in seconds) that the code can run.
    pub max_time: u32,
    /// Maximum memory (in bytes) that the code can use.
    pub max_memory: u32,
    /// Whether to save the output of the code.
    pub save_output: bool,
}

/// Result of running code.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Output of the code (if any).
    pub output: Option<String>,
    /// Exit code of the code.
    pub exit_code: i32,
}