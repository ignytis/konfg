pub mod file;
pub mod stdio;

use std::{collections::VecDeque, sync::LazyLock};

use anyhow::{Result, anyhow};

use crate::cli::IoSpec;

const REGISTERED_HANDLERS: LazyLock<Vec<Box<dyn IoHandler>>> =
    LazyLock::new(|| vec![Box::new(stdio::StdioHandler), Box::new(file::FileHandler)]);

/// Trait for handling input/output operations.
pub trait IoHandler: Send + Sync {
    /// Reads raw content from the source.
    fn read(&self, source: &str) -> Result<String>;

    /// Writes serialized content to the destination.
    fn write(&self, dest: &str, content: &str) -> Result<()>;

    /// Checks if this handler supports the given kind, e.g. "file" or "stdio".
    fn supports(&self, kind: &str) -> bool;

    /// Clones the handler into a boxed trait object.
    fn clone_box(&self) -> Box<dyn IoHandler>;

    /// Attempts to pop tokens from `tokens` and construct an `IoSpec`.
    /// Returns `None` if the first token is not supported by this handler.
    fn try_parse_spec(&self, tokens: &mut VecDeque<String>) -> Option<Result<IoSpec>>;
}

impl Clone for Box<dyn IoHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Factory method to get the appropriate IO handler for the given kind ("file" or "stdio").
pub fn get_handler(kind: &str) -> Result<Box<dyn IoHandler>> {
    for handler in REGISTERED_HANDLERS.iter() {
        if handler.supports(kind) {
            return Ok(handler.clone());
        }
    }

    Err(anyhow!("No IO handler found for: {}", kind))
}

/// Parses a flat list of tokens into a `Vec<IoSpec>` by delegating to registered handlers.
pub fn parse_specs(tokens: Vec<String>) -> Result<Vec<IoSpec>> {
    let mut queue: VecDeque<String> = tokens.into();
    let mut specs = Vec::new();

    while !queue.is_empty() {
        println!("Queue: {:?}", &queue);
        let mut matched = false;
        for handler in REGISTERED_HANDLERS.iter() {
            if let Some(result) = handler.try_parse_spec(&mut queue) {
                specs.push(result?);
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(anyhow!("Unrecognized input token: {:?}", queue.front()));
        }
    }

    Ok(specs)
}
