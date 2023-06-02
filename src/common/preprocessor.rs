use std::{fmt::Debug, io::Read, sync::Arc};

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

impl<F> Preprocessor for F
where
    F: Fn(&str) -> PreprocessorResult<String> + Send + Sync + Clone,
{
    fn preprocess(&self, code: &str) -> PreprocessorResult<String> {
        self(code)
    }
}

/// Bundle of preprocessors. It preprocesses code using all preprocessors in the bundle.
/// It can be used to combine multiple preprocessors into one.
#[derive(Clone)]
pub struct PreprocessorBundle {
    pub(crate) preprocessors: Vec<Arc<dyn Preprocessor>>,
}

impl Default for PreprocessorBundle {
    fn default() -> Self {
        Self::new()
    }
}

impl PreprocessorBundle {
    /// Creates new preprocessor bundle.
    pub fn new() -> Self {
        Self {
            preprocessors: Vec::new(),
        }
    }

    /// Adds preprocessor to the bundle.
    pub fn add_preprocessor(mut self, preprocessor: impl Preprocessor + 'static) -> Self {
        self.preprocessors.push(Arc::new(preprocessor));
        self
    }

    /// Preprocesses code using all preprocessors in the bundle.
    pub fn preprocess(&self, code: &mut impl Read) -> String {
        let mut code = std::io::read_to_string(code).unwrap();

        for preprocessor in &self.preprocessors {
            code = match preprocessor.preprocess(&code) {
                Ok(code) => code,
                Err(err) => panic!("Preprocessor error: {:?}", err),
            };
        }

        code
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_preprocessor_bundle() {
        use super::*;

        let bundle = PreprocessorBundle::new()
            .add_preprocessor(|code: &str| Ok(code.replace("a", "b")))
            .add_preprocessor(|code: &str| Ok(code.replace("b", "c")));

        let code = "a";
        let code = bundle.preprocess(&mut code.as_bytes());
        assert_eq!(code, "c");
    }
}
