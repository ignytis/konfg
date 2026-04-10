pub mod file;
pub mod stdio;

use anyhow::{anyhow, Result};

/// Trait for handling input/output operations.
pub trait IoHandler {
    /// Reads raw content from the source.
    fn read(&self, source: &str) -> Result<String>;

    /// Writes serialized content to the destination.
    fn write(&self, dest: &str, content: &str) -> Result<()>;

    /// Checks if this handler supports the given scheme
    /// For example: "stdio-yaml", "file-toml"
    fn supports(&self, scheme: &str) -> bool;
}

/// Factory method to get the appropriate IO handler for the given scheme.
/// Iterates over all registered handlers and returns the first one that supports the kind.
pub fn get_handler(scheme: &str) -> Result<Box<dyn IoHandler>> {
    let handlers: Vec<Box<dyn IoHandler>> =
        vec![Box::new(stdio::StdioHandler), Box::new(file::FileHandler)];

    for handler in handlers {
        if handler.supports(scheme) {
            return Ok(handler);
        }
    }

    Err(anyhow!("No IO handler found for: {}", scheme))
}
