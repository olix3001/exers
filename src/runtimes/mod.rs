//! Runtimes for running code.
//! 
//! Currently available runtimes are:
//! - [Native](native_runtime)
//! - [WASM](wasm_runtime)

use crate::compilers::CompiledCode;
use std::fmt::Debug;

#[cfg(all(feature = "jailed", feature = "native"))]
pub mod jailed_runtime;
#[cfg(feature = "native")]
pub mod native_runtime;
#[cfg(feature = "wasm")]
pub mod wasm_runtime;

/// Trait for every code runtime.
/// Represents a runtime that can be used to run some code.
pub trait CodeRuntime: Send + Sync + Sized {
    /// Configuration for the runtime.
    type Config: Send + Sync + Sized + Debug + Clone + Default;
    /// Additional compilation data.
    /// This is used to pass additional data from the compiler to the runtime.
    type AdditionalData: Send + Sync + Sized + Debug + Clone + Default;
    /// Error type for the runtime.
    type Error: Send + Sync + Sized + 'static;

    /// Run compiled code. Returns saved output (if any) and exit code.
    fn run(
        &self,
        code: &CompiledCode<Self>,
        config: Self::Config,
    ) -> Result<ExecutionResult, Self::Error>;
}

/// Result of running code.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Output of the code (if any).
    pub stdout: Option<String>,
    /// Error of the code (if any).
    pub stderr: Option<String>,
    /// Time taken by the code to run.
    pub time_taken: std::time::Duration,
    /// Exit code of the code.
    pub exit_code: i32,
}
