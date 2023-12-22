use thiserror::Error;
use crate::{CompilationError, MarkdownPart};
use crate::context::Context;
use crate::types::{Function, Value};

pub trait Plugin {
    fn exposed_functions(&self) -> &'static [Function];
    fn function_called(&self, function: &str, arguments: Vec<Value>, ctx: Context) -> Result<Vec<MarkdownPart>, PluginError>;
}
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Function `{0}` is not found.")]
    FunctionNotFound(String),
    #[error("Invalid arguments")]
    InvalidArguments,
    #[error("External error: `{0}`")]
    ExternalError(String),
    #[error("Nested compilation error: `{0}`")]
    CompilationError(Box<CompilationError>),
}

