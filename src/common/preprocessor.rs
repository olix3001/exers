use std::fmt::Debug;

/// Errors that can occur while preprocessing code.
#[derive(Debug, Clone)]
pub enum PreprocessorError {
    /// Parser error.
    ParserError(String),

    /// Error while preprocessing code.
    Other(String),
    // TODO: Add more errors.
}

pub type PreprocessorResult<T> = Result<T, PreprocessorError>;

/// Preprocessor trait. Preprocessors are used to change the code before compilation.
pub trait Preprocessor: Send + Sync {
    /// Preprocesses code. It can change the code, or return an error.
    fn preprocess(&self, code: &str) -> PreprocessorResult<String>;
}
