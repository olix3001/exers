use std::{io, fmt::Debug, marker::PhantomData, path::PathBuf, sync::{Arc, Mutex}};

use tempfile::TempDir;

use crate::runtimes::CodeRuntime;

pub mod rust;

/// Trait for every compiler that can be used to compile some code.
pub trait Compiler<R: CodeRuntime>: Send + Sync + Sized {
    /// Configuration for the compiler.
    type Config: Send + Sync + Sized + Debug + Clone + Default + IntoArgs;

    /// Compile the given code (as stream of bytes) and return the executable (in temporary file).
    fn compile(&self, code: &mut impl io::Read, config: CompilerConfig<R, Self>) -> io::Result<CompiledCode<Self, R>>;
}

/// Configuration for compilers.
#[derive(Debug, Clone)]
pub struct CompilerConfig<R: CodeRuntime, C: Compiler<R>> {
    /// Marker for the runtime used to run the compiled code.
    pub runtime_marker: PhantomData<R>,
    /// Compiler specific configuration.
    /// This is used to pass additional arguments to the compiler.
    pub compiler_specific: C::Config
}

impl<R: CodeRuntime, C: Compiler<R>> Default for CompilerConfig<R, C> {
    fn default() -> Self {
        CompilerConfig {
            runtime_marker: Default::default(),
            compiler_specific: Default::default()
        }
    }
}

/// Compiled code (executable).
/// Represents compiled code with additional information.
#[derive(Debug, Clone)]
pub struct CompiledCode<C: Compiler<R>, R: CodeRuntime> {
    /// Executable file (in temporary file).
    pub executable: Option<PathBuf>,
    /// Configuration used to compile the code.
    pub config: CompilerConfig<R, C>,    

    /// Compiler used to compile the code (marker) 
    pub compiler_marker: PhantomData<C>,

    /// Handle to the temporary directory.
    /// This is used to clean up the temporary directory when this object is dropped.
    temp_dir_handle: Arc<Mutex<Option<TempDir>>>
}

impl <C: Compiler<R>, R: CodeRuntime> CompiledCode<C, R> {
    /// Clean up the compiled code.
    /// This deletes the temporary directory containing the executable.
    pub fn clean_up(&mut self) -> io::Result<()> {
        // Delete the temporary directory.
        let temp_dir = self.temp_dir_handle.lock().unwrap().take().unwrap();
        temp_dir.close()?;

        Ok(())
    }
}

impl <C: Compiler<R>, R: CodeRuntime> Drop for CompiledCode<C, R> {
    fn drop(&mut self) {
        self.clean_up().unwrap();
    }
}

// Converts Config to args.
pub trait IntoArgs {
    fn into_args(self) -> Vec<String>;
}