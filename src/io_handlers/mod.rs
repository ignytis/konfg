pub mod file;
pub mod stdio;

use std::sync::LazyLock;

use anyhow::{anyhow, Result};

const REGISTERED_HANDLERS: LazyLock<Vec<Box<dyn IoHandler>>> =
    LazyLock::new(|| vec![Box::new(stdio::StdioHandler), Box::new(file::FileHandler)]);

/// Trait for handling input/output operations.
pub trait IoHandler: Send + Sync {
    /// Reads raw content from the source.
    fn read(&self, source: &str) -> Result<String>;

    /// Writes serialized content to the destination.
    fn write(&self, dest: &str, content: &str) -> Result<()>;

    /// Checks if this handler supports the given scheme
    /// For example: "stdio-yaml", "file-toml"
    fn supports(&self, scheme: &str) -> bool;

    /// Clones the handler into a boxed trait object.
    fn clone_box(&self) -> Box<dyn IoHandler>;
}

impl Clone for Box<dyn IoHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Factory method to get the appropriate IO handler for the given scheme.
/// Iterates over all registered handlers and returns the first one that supports the kind.
pub fn get_handler(scheme: &str) -> Result<Box<dyn IoHandler>> {
    for handler in REGISTERED_HANDLERS.iter() {
        if handler.supports(scheme) {
            return Ok(handler.clone());
        }
    }

    Err(anyhow!("No IO handler found for: {}", scheme))
}
