use std::{error::Error, fmt::Display};

/// Enum for opt level
/// Some compilers may not support all opt levels
#[derive(Debug, Clone)]
pub enum OptLevel {
    /// No optimizations
    None,
    /// Optimize for speed
    Speed,
    /// Optimize for size
    Size,
    /// Opt level 1
    O1,
    /// Opt level 2
    O2,
    /// Opt level 3
    O3,
    /// Custom optimization level
    Custom(String),
}

impl OptLevel {
    pub fn as_stanard_opt_char(&self) -> String {
        match self {
            OptLevel::None => "0",
            OptLevel::Speed => "fast",
            OptLevel::Size => "z",
            OptLevel::O1 => "1",
            OptLevel::O2 => "2",
            OptLevel::O3 => "3",
            OptLevel::Custom(c) => c,
        }
        .to_string()
    }
}

/// Checks if program is installed and panic with nice message if it is not.
pub fn check_program_installed(program: &str) -> Result<(), CompilationError> {
    if !which::which(program).is_ok() {
        Err(CompilationError::ProgramNotInstalled(program.to_string()))
    } else {
        Ok(())
    }
}

/// Error for compiler.
#[derive(Debug)]
pub enum CompilationError {
    /// IO error.
    /// This is returned when there is an error while reading or writing to file.
    IoError(std::io::Error),

    /// Error while compiling.
    /// This is returned when compiler returns non-zero exit code.
    /// This contains stderr of compiler.
    CompilationFailed(String),

    /// Program is not installed.
    /// This is returned when compiler dependency is not installed.
    ProgramNotInstalled(String),
}

impl From<std::io::Error> for CompilationError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

/// Type for convinient result of compiler.
pub type CompilationResult<T> = Result<T, CompilationError>;

impl Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationError::IoError(e) => write!(f, "IO error: {}", e),
            CompilationError::CompilationFailed(e) => write!(f, "Compilation failed: {}", e),
            CompilationError::ProgramNotInstalled(e) => write!(f, "Program not installed: {}", e),
        }
    }
}
impl Error for CompilationError {}
