use crate::compilers::{Compiler, CompiledCode};

use super::{CodeRuntime, ExecutionResult};

#[derive(Debug, Clone)]
pub struct WasmRuntime;

/// Runtime for wasm code.
impl CodeRuntime for WasmRuntime {
    fn run<C: Compiler<Self>>(code: CompiledCode<C, Self>) -> ExecutionResult {
        todo!()
    }
}   