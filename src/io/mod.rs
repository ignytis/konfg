pub mod file;
pub mod stdio;

use anyhow::{anyhow, Result};

/// Trait for handling input/output operations.
pub trait IoHandler {
    /// Reads raw content from the source.
    fn read(&self, source: &str) -> Result<String>;

    /// Writes serialized content to the destination.
    fn write(&self, dest: &str, content: &str) -> Result<()>;

    /// Checks if this handler supports the given I/O kind (scheme or path).
    /// For example: "stdin", "stdout", "file", or a file path.
    fn supports(&self, io_kind: &str) -> bool;
}

/// Factory method to get the appropriate IO handler for the given I/O kind.
/// Iterates over all registered handlers and returns the first one that supports the kind.
pub fn get_handler(io_kind: &str) -> Result<Box<dyn IoHandler>> {
    let handlers: Vec<Box<dyn IoHandler>> = vec![
        Box::new(stdio::StdioHandler),
        Box::new(file::FileHandler),
    ];

    for handler in handlers {
        if handler.supports(io_kind) {
            return Ok(handler);
        }
    }

    Err(anyhow!(
        "No IO handler found for: {}",
        io_kind
    ))
}
