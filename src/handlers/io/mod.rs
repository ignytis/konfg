pub mod env;
pub mod file;
pub mod stdio;

use std::{collections::VecDeque, sync::LazyLock};

use anyhow::{anyhow, Result};

use crate::types::endpoint::Endpoint;

const REGISTERED_HANDLERS: LazyLock<Vec<Box<dyn IoHandler>>> = LazyLock::new(|| {
    vec![
        Box::new(stdio::StdioHandler),
        Box::new(file::FileHandler),
        Box::new(env::EnvHandler),
    ]
});

/// Result of attempting to parse tokens by a handler.
pub enum TryParseResult {
    /// Arguments were successfully parsed into Endpoint
    Success(Endpoint),
    /// Parser does not support the provided input
    NotSupported,
    /// Parser supports the provided input, but an error occurred
    Error(anyhow::Error),
}

/// Trait for handling input/output operations.
pub trait IoHandler: Send + Sync {
    /// Reads raw content from the source.
    fn read(&self, path: Option<&str>) -> Result<String>;

    /// Writes serialized content to the destination.
    fn write(&self, content: &str, path: Option<&str>) -> Result<()>;

    /// Checks if this handler supports the given kind, e.g. "file" or "stdio".
    fn supports(&self, kind: &str) -> bool;

    /// Clones the handler into a boxed trait object.
    fn clone_box(&self) -> Box<dyn IoHandler>;

    /// Attempts to pop tokens from `tokens` and construct an `Endpoint`.
    /// Returns `TryParseResult::NotSupported` if the first token is not supported by this handler.
    fn try_parse_tokens(&self, tokens: &mut VecDeque<String>) -> TryParseResult;
}

impl Clone for Box<dyn IoHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Parses a flat list of tokens into an `Endpoint` using registered handlers.
/// `tokens` is a VecDeque of string parameters for single input / output.
/// Example: ['file', '/path/to/file.cfg', 'yaml']
pub fn parse_tokens(mut tokens: VecDeque<String>) -> Result<Endpoint> {
    for io_handler in REGISTERED_HANDLERS.iter() {
        match io_handler.try_parse_tokens(&mut tokens) {
            TryParseResult::Success(e) => return Ok(e),
            TryParseResult::NotSupported => continue,
            TryParseResult::Error(e) => return Err(e),
        }
    }

    return Err(anyhow!("Unrecognized input token: {:?}", tokens));
}
