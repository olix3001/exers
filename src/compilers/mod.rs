use std::{
    fmt::Debug,
    io,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tempfile::TempDir;

use crate::runtimes::CodeRuntime;

#[cfg(feature = "cpp")]
pub mod cpp_compiler;
#[cfg(feature = "rust")]
pub mod rust_compiler;

/// Trait for every compiler that can be used to compile some code.
pub trait Compiler<R: CodeRuntime>: Send + Sync + Sized {
    /// Configuration for the compiler.
    type Config: Send + Sync + Sized + Debug + Clone + Default + IntoArgs;

    /// Compile the given code (as stream of bytes) and return the executable (in temporary file).
    fn compile(
        &self,
        code: &mut impl io::Read,
        config: Self::Config,
    ) -> io::Result<CompiledCode<R>>;
}

/// Compiled code (executable).
/// Represents compiled code with additional information.
#[derive(Debug, Clone)]
pub struct CompiledCode<R: CodeRuntime> {
    /// Executable file (in temporary file).
    pub executable: Option<PathBuf>,

    /// Handle to the temporary directory.
    /// This is used to clean up the temporary directory when this object is dropped.
    pub temp_dir_handle: Arc<Mutex<Option<TempDir>>>,

    /// Additional data for the runtime.
    /// This can differ for different runtimes.
    pub additional_data: R::AdditionalData,

    /// Runtime marker.
    pub runtime_marker: std::marker::PhantomData<R>,
}

impl<R: CodeRuntime> CompiledCode<R> {
    /// Clean up the compiled code.
    /// This deletes the temporary directory containing the executable.
    pub fn clean_up(&mut self) -> io::Result<()> {
        // Delete the temporary directory.
        match self.temp_dir_handle.lock().unwrap().take() {
            Some(temp_dir) => {
                temp_dir.close()?;
            }
            None => {}
        }

        Ok(())
    }
}

impl<R: CodeRuntime> Drop for CompiledCode<R> {
    fn drop(&mut self) {
        self.clean_up().unwrap();
    }
}

// Converts Config to args.
pub trait IntoArgs {
    fn into_args(self) -> Vec<String>;
}
