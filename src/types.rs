use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    String,
    Path,
    Number,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Path(PathBuf),
    Number(i64)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: &'static str,
    pub signatures: &'static [&'static [Type]]
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownPart {
    Text {
        content: String,
        source: PathBuf,
    },
    Call {
        function: String,
        arguments: Vec<Value>,
        source: PathBuf,
    },
}
