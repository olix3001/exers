use std::path::PathBuf;

/// Represents input data for the code.
#[derive(Debug, Clone)]
pub enum InputData {
    /// Stdin will be read from the given file.
    File(PathBuf),
    /// Stdin will be read from the given string.
    String(String),
    /// Stdin will be ignored.
    Ignore
}