use std::{io::Write, process::Command};

use crate::{
    common::{compiler::CompilationResult, runtime::InputData},
    compilers::{CompiledCode, Compiler},
};

use super::{native_runtime::NativeRuntime, CodeRuntime};

const JAIL: &[u8] = include_bytes!("../../assets/jail.sh");

/// Jailed runtime.
/// This uses chroot jail to run the code.
/// This is only available on Linux and requires root privileges.
/// It is automatically implemented for every native compiler.
#[derive(Debug, Clone)]
pub struct JailedRuntime;

/// Jail configuration.
#[derive(Debug, Clone, Default)]
pub struct JailedConfig {
    native_runtime_config: super::native_runtime::NativeConfig,
}

/// Error type for the runtime.
#[derive(Debug)]
pub enum JailedError {
    /// Error in chroot jail.
    IOError(std::io::Error),
    /// Root privileges are required to run chroot jail.
    RootRequired,
}

impl From<std::io::Error> for JailedError {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e)
    }
}

/// Runtime for jailed code.
impl CodeRuntime for JailedRuntime {
    /// Configuration for the runtime.
    type Config = JailedConfig;
    /// Additional compilation data.
    type AdditionalData = super::native_runtime::NativeAdditionalData;
    /// Error type for the runtime.
    type Error = JailedError;

    /// Runs the code in a chroot jail.
    fn run(
        &self,
        code: &crate::compilers::CompiledCode<Self>,
        config: Self::Config,
    ) -> Result<super::ExecutionResult, Self::Error> {
        // Check root
        if !check_root() {
            return Err(Self::Error::RootRequired);
        }

        // Get temporary directory.
        let temp_dir = code.executable.as_ref().unwrap().parent().unwrap();

        // Copy jail script to temporary directory.
        let jail_path = temp_dir.join("jail.sh");
        // Create file
        std::fs::File::create(&jail_path)?;
        std::fs::write(&jail_path, JAIL)?;

        // Run jail
        let mut command = Command::new("bash");
        command.arg(jail_path);
        command.arg(temp_dir.join("jail"));

        match &code.additional_data.program {
            Some(program) => {
                command.arg(which::which(program).unwrap());
                command.arg(code.executable.as_ref().unwrap());
            }
            None => {
                command.arg(code.executable.as_ref().unwrap());
            }
        }

        // Setup stdin.
        match config.native_runtime_config.stdin {
            InputData::Ignore => {
                command.stdin(std::process::Stdio::null());
            }
            _ => {
                command.stdin(std::process::Stdio::piped());
            }
        };

        // Setup stdout.
        command.stdout(std::process::Stdio::piped());
        // Setup stderr.
        command.stderr(std::process::Stdio::piped());

        // Spawn the command.
        let mut child = command.spawn()?;

        // Start timer.
        let start_time = std::time::Instant::now();

        // Write to stdin.
        match config.native_runtime_config.stdin {
            InputData::Ignore => {}
            InputData::String(data) => {
                child.stdin.as_mut().unwrap().write_all(data.as_bytes())?;
            }
            InputData::File(path) => {
                let mut file = std::fs::File::open(path)?;
                std::io::copy(&mut file, child.stdin.as_mut().unwrap())?;
            }
        };

        // Wait for the child to finish.
        let output = child.wait_with_output()?;

        // Stop timer.
        let time_taken = start_time.elapsed();

        // Get stdout.
        let stdout = match output.stdout.len() {
            0 => None,
            _ => Some(String::from_utf8(output.stdout).unwrap()),
        };

        // Get stderr.
        let stderr = match output.stderr.len() {
            0 => None,
            _ => Some(String::from_utf8(output.stderr).unwrap()),
        };

        // Return the result.
        Ok(super::ExecutionResult {
            stdout,
            stderr,
            time_taken,
            exit_code: output.status.code().unwrap_or(0),
        })
    }
}

/// Implementation of JailedRuntime compiler for every native compiler.
impl<C> Compiler<JailedRuntime> for C
where
    C: Compiler<NativeRuntime>,
{
    /// Configuration for the compiler.
    type Config = C::Config;

    /// Compiles the code using the native compiler.
    fn compile(
        &self,
        code: &mut impl std::io::Read,
        config: Self::Config,
    ) -> CompilationResult<crate::compilers::CompiledCode<JailedRuntime>> {
        let native_code: CompiledCode<NativeRuntime> = C::compile(self, code, config)?;
        // Without this somehow temp_dir disappears :p
        let temp_dir = native_code.temp_dir_handle.lock().unwrap().take().unwrap();
        let temp_dir_handle = std::sync::Arc::new(std::sync::Mutex::new(Some(temp_dir)));
        Ok(CompiledCode {
            executable: native_code.executable.clone(),
            temp_dir_handle,
            runtime_marker: std::marker::PhantomData,
            additional_data: native_code.additional_data.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compilers::{rust_compiler::RustCompiler, Compiler};

    #[test]
    fn test_run_jailed() {
        let code = r#"
        fn main() {
            println!("Hello, world!");
        }
        "#;

        let compiled_code = RustCompiler
            .compile(&mut code.as_bytes(), Default::default())
            .unwrap();
        let result = JailedRuntime
            .run(&compiled_code, Default::default())
            .unwrap();

        assert_eq!(result.stdout, Some("Hello, world!\n".to_string()));
    }
}

fn check_root() -> bool {
    #[cfg(target_family = "unix")]
    unsafe {
        libc::getuid() == 0
    }
    #[cfg(target_family = "windows")]
    false
}
