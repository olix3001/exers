use crate::compilers::{CompiledCode, Compiler};
use std::{fmt::Debug};

pub mod wasm_runtime;

/// Trait for every code runtime.
/// Represents a runtime that can be used to run some code.
pub trait CodeRuntime: Send + Sync + Sized {
    /// Configuration for the runtime.
    type Config: Send + Sync + Sized + Debug + Clone + Default;
    /// Error type for the runtime.
    type Error: Send + Sync + Sized + 'static;
    
    /// Run compiled code. Returns saved output (if any) and exit code.
    fn run<C: Compiler<Self>>(code: &CompiledCode<C, Self>, config: Self::Config) -> Result<ExecutionResult, Self::Error>;
}

/// Result of running code.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Output of the code (if any).
    pub stdout: Option<String>,
    /// Error of the code (if any).
    pub stderr: Option<String>, 
    /// Exit code of the code.
    pub exit_code: i32,
}