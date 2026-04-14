use anyhow::Result;
use serde_json::Value;

use crate::handlers::{
    format::{self as format_handlers, FormatHandler},
    io,
};

/// Represents a configuration source or destination with associated IO and format handlers.
pub struct Endpoint {
    io: Box<dyn io::IoHandler>,
    format: Box<dyn FormatHandler>,
    path: String,
}

impl Endpoint {
    /// Creates an `Endpoint` from an IO kind (e.g. `"file"`, `"stdio"`), a format name
    /// (e.g. `"yaml"`, `"json"`), and an optional path.
    pub fn new(kind: &str, format: &str, path: &str) -> Result<Self> {
        Ok(Self {
            io: io::get_handler(kind)?,
            format: format_handlers::get_handler(format)?,
            path: path.to_string(),
        })
    }

    /// Reads raw string content from this endpoint.
    pub fn read(&self) -> Result<String> {
        self.io.read(&self.path)
    }

    /// Serializes and writes a value to this endpoint.
    pub fn write(&self, value: &Value) -> Result<()> {
        let serialized = self.format.serialize(value)?;
        self.io.write(&self.path, &serialized)
    }

    /// Returns the format handler for this endpoint.
    pub fn format(&self) -> &dyn FormatHandler {
        self.format.as_ref()
    }
}
