pub mod file;
pub mod stdio;

use std::{collections::VecDeque, sync::LazyLock};

use anyhow::{anyhow, Result};

use crate::types::endpoint::Endpoint;

const REGISTERED_HANDLERS: LazyLock<Vec<Box<dyn IoHandler>>> =
    LazyLock::new(|| vec![Box::new(stdio::StdioHandler), Box::new(file::FileHandler)]);

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

    /// Attempts to pop tokens from `tokens` and construct an `IoSpec`.
    /// Returns `None` if the first token is not supported by this handler.
    fn try_parse_spec(&self, tokens: &mut VecDeque<String>) -> Result<Option<Endpoint>>;
}

impl Clone for Box<dyn IoHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Parses a flat list of tokens into a `Vec<Endpoint>` using registered handlers.
/// token = a list of string parameters for single input / output.
/// Example: ['file', '/path/to/file.cfg', 'yaml']
pub fn parse_tokens(tokens: &Vec<String>) -> Result<Endpoint> {
    let mut queue: VecDeque<String> = tokens.clone().into();
    // let mut specs = Vec::new();

    for io_handler in REGISTERED_HANDLERS.iter() {
        match io_handler.try_parse_spec(&mut queue) {
            Ok(opt_endpoint) => match opt_endpoint {
                Some(e) => return Ok(e),
                None => continue,
            },
            Err(e) => return Err(e),
        }
    }

    return Err(anyhow!("Unrecognized input token: {:?}", tokens));
}
