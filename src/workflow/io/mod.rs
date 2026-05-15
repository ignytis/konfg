pub mod env;
pub mod file;
pub mod stdio;

use std::{
    collections::{HashMap, VecDeque},
    sync::LazyLock,
};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::handlers::format::FormatHandler;

const REGISTERED_HANDLERS: LazyLock<Vec<Box<dyn IoHandler>>> = LazyLock::new(|| {
    vec![
        Box::new(stdio::StdioHandler),
        Box::new(file::FileHandler),
        Box::new(env::EnvHandler),
    ]
});

/// Represents a configuration source or destination with associated IO and format handlers.
pub struct Stage {
    pub io_handler: Box<dyn IoHandler>,
    pub format_handler: Option<Box<dyn FormatHandler>>,
    pub args: HashMap<String, String>,
}

impl Stage {
    pub fn new(
        io_handler: Box<dyn IoHandler>,
        format_handler: Option<Box<dyn FormatHandler>>,
        args: HashMap<String, String>,
    ) -> Self {
        Self {
            io_handler,
            format_handler,
            args,
        }
    }

    /// Reads raw string content from this stage.
    pub fn read(&self) -> Result<String> {
        self.io_handler.read(&self.args)
    }

    /// Writes serialized content to this stage.
    pub fn write(&self, value: &Value) -> Result<()> {
        let serialized_value = match &self.format_handler {
            Some(h) => h.serialize(value)?,
            None => value.to_string(),
        };
        self.io_handler.write(&serialized_value, &self.args)
    }

    pub fn parse(&self, content: &str) -> Result<Value> {
        match &self.format_handler {
            Some(h) => h.parse(content),
            None => Err(anyhow!(
                "Inputs/outputs without defined formats are not supported"
            )),
        }
    }
}

/// Result of attempting to parse tokens by a handler.
pub enum TryParseResult {
    /// Arguments were successfully parsed into Stage
    Success(Stage),
    /// Parser does not support the provided input
    NotSupported,
    /// Parser supports the provided input, but an error occurred
    Error(anyhow::Error),
}

/// Trait for handling input/output operations.
pub trait IoHandler: Send + Sync {
    /// Reads raw content from the source.
    fn read(&self, args: &HashMap<String, String>) -> Result<String>;

    /// Writes serialized content to the destination.
    fn write(&self, content: &str, args: &HashMap<String, String>) -> Result<()>;

    /// Checks if this handler supports the given kind, e.g. "file" or "stdio".
    fn supports(&self, kind: &str) -> bool;

    /// Clones the handler into a boxed trait object.
    fn clone_box(&self) -> Box<dyn IoHandler>;

    /// Attempts to pop tokens from `tokens` and construct a `Stage`.
    /// Returns `TryParseResult::NotSupported` if the first token is not supported by this handler.
    fn try_parse_tokens(&self, tokens: &mut VecDeque<String>) -> TryParseResult;
}

impl Clone for Box<dyn IoHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Parses a flat list of tokens into a `Stage` using registered handlers.
/// `tokens` is a VecDeque of string parameters for single input / output.
/// Example: ['file', '/path/to/file.cfg', 'yaml']
pub fn parse_tokens(mut tokens: VecDeque<String>) -> Result<Stage> {
    for io_handler in REGISTERED_HANDLERS.iter() {
        match io_handler.try_parse_tokens(&mut tokens) {
            TryParseResult::Success(s) => return Ok(s),
            TryParseResult::NotSupported => continue,
            TryParseResult::Error(e) => return Err(e),
        }
    }

    return Err(anyhow!("Unrecognized input token: {:?}", tokens));
}
