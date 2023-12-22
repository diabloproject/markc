mod types;
mod context;
mod plugin;
mod plugins;

use crate::types::*;
use clap::Parser;
use std::path::{Path, PathBuf};
use thiserror::Error;
use crate::context::Context;
use crate::plugin::{Plugin, PluginError};

#[derive(Parser, Debug, Clone)]
struct Args {
    file: PathBuf,
    #[arg(short, long, default_value = "dist.md")]
    output: PathBuf,
}

#[derive(Debug, Error)]
pub enum CompilationError {
    #[error("")]
    PluginError(PluginError),
    #[error("")]
    IOError(std::io::Error),
    #[error("")]
    CallParseError(CallParseError),
}

impl From<std::io::Error> for CompilationError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}



fn parse_md(path: &Path) -> Result<Vec<MarkdownPart>, CompilationError> {
    let content = std::fs::read_to_string(path)?;
    let mut parts = vec![];
    enum CurrentState {
        InText,
        InCall,
    }
    let mut buf = String::new();
    let mut cs = CurrentState::InText;

    for c in content.chars() {
        match cs {
            CurrentState::InText => {
                if c == '{' && buf.ends_with('{') {
                    buf.pop();
                    parts.push(MarkdownPart::Text {
                        content: buf,
                        source: path.into(),
                    });
                    cs = CurrentState::InCall;
                    buf = String::new();
                } else {
                    buf.push(c);
                }
            }
            CurrentState::InCall => {
                if c == '}' && buf.ends_with('}') {
                    buf.pop();
                    let (function, arguments) = parse_call(&buf)?;
                    parts.push(MarkdownPart::Call {
                        function,
                        arguments,
                        source: path.into(),
                    });
                    cs = CurrentState::InText;
                    buf = String::new();
                } else {
                    buf.push(c)
                }
            }
        }
    }
    parts.push(MarkdownPart::Text {
        content: buf,
        source: path.into(),
    });
    Ok(parts)
}

#[derive(Debug, Error)]
pub enum CallParseError {
    #[error("")]
    InvalidSymbol(char),
    #[error("")]
    EmptyArgument,
    #[error("")]
    UnclosedLiteral,
    #[error("")]
    ParseIntError(std::num::ParseIntError),
}

impl From<CallParseError> for CompilationError {
    fn from(value: CallParseError) -> Self {
        CompilationError::CallParseError(value)
    }
}

fn parse_arg(buf: &str) -> Result<Value, CallParseError> {
    let a = buf.trim();
    match a.chars().next().ok_or(CallParseError::EmptyArgument)? {
        '#' => {
            if !a.ends_with('#') {
                Err(CallParseError::UnclosedLiteral)
            } else {
                let path = a.strip_prefix('#').unwrap().strip_suffix('#').unwrap();
                Ok(Value::Path(path.into()))
            }
        }
        '"' => {
            if !a.ends_with('"') {
                Err(CallParseError::UnclosedLiteral)
            } else {
                let s = a.strip_prefix('"').unwrap().strip_suffix('"').unwrap();
                Ok(Value::String(s.into()))
            }
        }
        _ => {
            Ok(Value::Number(a.parse().map_err(|err| CallParseError::ParseIntError(err))?))
        }
    }
}

fn parse_call(buffer: &str) -> Result<(String, Vec<Value>), CallParseError> {
    enum CurrentState {
        Start,
        FunctionName,
        FunctionArgs,
    }
    let mut in_path = false;
    let mut in_string = false;
    let mut cs = CurrentState::Start;
    let mut function_name: String = String::new();
    let mut args: Vec<Value> = vec![];
    let mut buf: String = String::new();
    let mut i = 0;
    while i < buffer.len() {
        let c = buffer.chars().skip(i).next().unwrap();
        match cs {
            CurrentState::Start => {
                if !c.is_whitespace() {
                    cs = CurrentState::FunctionName;
                    i -= 1;
                }
            }
            CurrentState::FunctionName => {
                if c.is_alphanumeric() {
                    buf.push(c);
                } else if c.is_whitespace() {} else if c == '(' {
                    function_name = buf;
                    buf = String::new();
                    cs = CurrentState::FunctionArgs;
                } else {
                    return Err(CallParseError::InvalidSymbol(c));
                }
            }
            CurrentState::FunctionArgs => {
                if c == '"' && !in_path {
                    in_string = !in_string;
                }
                if c == '#' && !in_string {
                    in_path = !in_path;
                }
                if in_path || in_string {
                    buf.push(c)
                } else if c == ')' {
                    args.push(parse_arg(&buf)?);
                    buf.clear();
                    break;
                } else if c == ',' {
                    args.push(parse_arg(&buf)?);
                    buf.clear();
                } else {
                    buf.push(c);
                }
            }
        }
        i += 1;
    }
    Ok((function_name, args))
}

fn evaluate(content: Vec<MarkdownPart>, plugins: &[Box<dyn Plugin>]) -> Result<Vec<MarkdownPart>, PluginError> {
    let mut new_parts = vec![];
    for part in content.into_iter() {
        match part {
            MarkdownPart::Text { .. } => {
                new_parts.push(part)
            }
            MarkdownPart::Call { function, arguments, source } => {
                let ctx = Context {
                    path: source.parent().unwrap().into()
                };
                for pl in plugins.iter() {
                    if pl.exposed_functions().iter().map(|f| f.name).collect::<Vec<_>>().contains(&function.as_str()) {
                        let parts = evaluate(pl.function_called(&function, arguments, ctx.clone())?, plugins)?;
                        new_parts.extend(parts.into_iter());
                        break;
                    }
                }
            }
        }
    };
    return Ok(new_parts);
}

fn rebuild(content: Vec<MarkdownPart>) -> String {
    let mut new_content: String = String::new();
    for part in content {
        match part {
            MarkdownPart::Text { content, .. } => {
                new_content.push_str(&content);
            }
            MarkdownPart::Call { .. } => { panic!("Call in rebuild") }
        }
    }
    return new_content;
}

fn main() {
    let args: Args = Args::parse();
    let content = parse_md(&args.file).unwrap();
    let content = evaluate(content, &[Box::new(plugins::include::IncludePlugin::new())]).unwrap();
    let content = rebuild(content);
    std::fs::write(args.output, content).unwrap();
}
