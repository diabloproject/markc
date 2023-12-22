use crate::context::Context;
use crate::{MarkdownPart, parse_md};
use crate::plugin::{ Plugin, PluginError };
use crate::types::*;

pub struct IncludePlugin;

impl IncludePlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for IncludePlugin {
    fn exposed_functions(&self) -> &'static [Function] {
        &[
            Function {
                name: "include",
                signatures: &[&[Type::Path]],
            }
        ]
    }

    fn function_called(&self, function: &str, arguments: Vec<Value>, ctx: Context) -> Result<Vec<MarkdownPart>, PluginError> {
        match function {
            "include" => {
                match arguments.first() {
                    None => { Err(PluginError::InvalidArguments) }
                    Some(x) => {
                        match x {
                            Value::Path(path) => {
                                let path = ctx.path.join(path);
                                parse_md(&path)
                                    .map_err(|err| PluginError::CompilationError(Box::new(err)))
                            }
                            _ => Err(PluginError::InvalidArguments)
                        }
                    }
                }
            }
            _ => Err(PluginError::FunctionNotFound(function.into()))
        }
    }
}
