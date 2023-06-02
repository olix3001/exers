//! Implements builder pattern for exers.

use std::{io::Read, ops::Deref};

use crate::{
    compilers::{CompiledCode, Compiler},
    runtimes::{CodeRuntime, ExecutionResult},
};

use super::{
    compiler::{CompilationError, CompilationResult},
    preprocessor::Preprocessor,
};

/// Builder for creating more complex compilers.
///
/// # Example
/// ```rs, no_run
/// // Imports
///
/// fn main() {
///    // Create new runtime.
///    let scratch_runtime = RuntimeBuilder::new()
///        .preprocessor(Scratch3ToCppPreprocessor::new())
///        .compiler(CppCompiler, None) // Compiler config is not needed.
///        .runtime(NativeRuntime, None) // Runtime config is not needed.
///        .build()
///
///     // Run code using preconfigured runtime.
///     println!("{:?}", scratch_runtime(&mut "code"));
/// }
pub struct RuntimeBuilder<C: Compiler<R>, R: CodeRuntime> {
    /// Preprocessors that will be used to preprocess code.
    preprocessors: Vec<Box<dyn Preprocessor>>,
    /// Compiler that will be used to compile code.
    compiler: Option<C>,
    /// Runtime that will be used to run code.
    runtime: Option<R>,

    /// Config for compiler.
    compiler_config: Option<C::Config>,
    /// Config for runtime.
    runtime_config: Option<R::Config>,
}

/// Errors that can occur while building compiler.
#[derive(Debug, Clone)]
pub enum RuntimeBuilderError {
    /// Compiler is not set.
    CompilerNotSet,
    /// Runtime is not set.
    RuntimeNotSet,
}

type RuntimeBuilderResult<T> = Result<T, RuntimeBuilderError>;

impl<C: Compiler<R> + 'static, R: CodeRuntime + 'static> RuntimeBuilder<C, R> {
    /// Creates new builder.
    pub const fn new() -> Self {
        Self {
            preprocessors: Vec::new(),
            compiler: None,
            runtime: None,
            compiler_config: None,
            runtime_config: None,
        }
    }

    /// Adds preprocessor to the builder.
    pub fn preprocessor(mut self, preprocessor: impl Preprocessor + 'static) -> Self {
        self.preprocessors.push(Box::new(preprocessor));
        self
    }

    /// Sets compiler to the builder.
    pub fn compiler(mut self, compiler: C, config: Option<C::Config>) -> Self {
        self.compiler = Some(compiler);
        self.compiler_config = config;
        self
    }

    /// Sets runtime to the builder.
    pub fn runtime(mut self, runtime: R, config: Option<R::Config>) -> Self {
        self.runtime = Some(runtime);
        self.runtime_config = config;
        self
    }

    /// Builds new compiler from builder.
    pub fn build(mut self) -> RuntimeBuilderResult<CustomRuntime<R>> {
        // Take compiler and runtime from builder.
        let compiler = self
            .compiler
            .take()
            .ok_or(RuntimeBuilderError::CompilerNotSet)?;
        let runtime = self
            .runtime
            .take()
            .ok_or(RuntimeBuilderError::RuntimeNotSet)?;

        // Take their configs, or use default if they are not set.
        let compiler_config = self.compiler_config.take().unwrap_or_default();
        let runtime_config = self.runtime_config.take().unwrap_or_default();

        // Compilation function
        let cf = move |code: &mut dyn std::io::Read| -> CompilationResult<CompiledCode<R>> {
            let mut code = std::io::BufReader::new(code);
            let mut code_str = String::new();
            code.read_to_string(&mut code_str).unwrap();
            let mut code = code_str;

            for preprocessor in self.preprocessors.iter() {
                code = preprocessor.preprocess(&code)?;
            }

            let compiled_code = compiler.compile(&mut code.as_bytes(), compiler_config.clone())?;
            Ok(compiled_code)
        };

        // Runtime function
        let rf = move |compiled_code: &CompiledCode<R>| -> Result<ExecutionResult, R::Error> {
            runtime.run(compiled_code, runtime_config.clone())
        };

        Ok(CustomRuntime::new(cf, rf))
    }
}

pub struct CustomRuntime<R: CodeRuntime> {
    /// Combination of compiler and runtime.
    #[allow(clippy::type_complexity)]
    crf: Box<dyn Fn(&mut dyn std::io::Read) -> Result<ExecutionResult, CustomRuntimeError<R>>>,
}

impl<R: CodeRuntime> CustomRuntime<R> {
    /// Creates new custom runtime. This should be used only by builder.
    #[allow(clippy::type_complexity)]
    pub(crate) fn new(
        cf: impl Fn(&mut dyn std::io::Read) -> CompilationResult<CompiledCode<R>> + 'static,
        rf: impl Fn(&CompiledCode<R>) -> Result<ExecutionResult, R::Error> + 'static,
    ) -> Self {
        Self {
            crf: Box::new(move |code| {
                let compiled_code =
                    cf(code).map_err(|e| CustomRuntimeError::CompilationError(e))?;
                (rf)(&compiled_code).map_err(|e| CustomRuntimeError::RuntimeError(e))
            }),
        }
    }

    /// Compiles and runs code using custom compiler and runtime.
    pub fn run(
        &self,
        code: &mut dyn std::io::Read,
    ) -> Result<ExecutionResult, CustomRuntimeError<R>> {
        (self.crf)(code)
    }
}

#[allow(clippy::type_complexity)]
impl<R: CodeRuntime + 'static> Deref for CustomRuntime<R> {
    type Target = dyn Fn(&mut dyn std::io::Read) -> Result<ExecutionResult, CustomRuntimeError<R>>;

    fn deref(&self) -> &Self::Target {
        &self.crf
    }
}

/// Error from either compiler or runtime.
#[derive(Debug)]
pub enum CustomRuntimeError<R: CodeRuntime> {
    /// Error from compiler.
    CompilationError(CompilationError),
    /// Error from runtime.
    RuntimeError(R::Error),
}

impl<R: CodeRuntime> From<CompilationError> for CustomRuntimeError<R> {
    fn from(error: CompilationError) -> Self {
        Self::CompilationError(error)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "wasm")]
    use crate::{compilers::rust_compiler::RustCompiler, runtimes::wasm_runtime::WasmRuntime};

    use super::RuntimeBuilder;

    #[test]
    #[cfg(feature = "wasm")]
    fn test_builder_rust_wasm() {
        let rust_wasm_runtime = RuntimeBuilder::new()
            .compiler(RustCompiler, None)
            .runtime(WasmRuntime, None)
            .build()
            .unwrap();

        let code = r#"
            fn main() {
                println!("Hello, world!");
            }
        "#;

        assert_eq!(
            rust_wasm_runtime(&mut code.as_bytes()).unwrap().stdout,
            Some("Hello, world!\n".to_string())
        );

        let code = r#"
            fn main() {
                println!("Hello, world!");
                println!("Hello, world!");
            }
        "#;

        assert_eq!(
            rust_wasm_runtime(&mut code.as_bytes()).unwrap().stdout,
            Some("Hello, world!\nHello, world!\n".to_string())
        );
    }
}
