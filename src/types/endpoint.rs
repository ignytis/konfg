use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::handlers::{format::FormatHandler, io};

/// Represents a configuration source or destination with associated IO and format handlers.
pub struct Endpoint {
    io_handler: Box<dyn io::IoHandler>,
    format_handler: Option<Box<dyn FormatHandler>>,
    path: Option<String>,
}

impl Endpoint {
    pub fn new(
        io_handler: Box<dyn io::IoHandler>,
        format_handler: Option<Box<dyn FormatHandler>>,
        path: Option<String>,
    ) -> Self {
        Self {
            io_handler,
            format_handler,
            path,
        }
    }

    /// Reads raw string content from this endpoint.
    pub fn read(&self) -> Result<String> {
        self.io_handler.read(self.path.as_ref().map(String::as_str))
    }

    /// Serializes and writes a value to this endpoint.
    pub fn write(&self, value: &Value) -> Result<()> {
        let serialized_value = match &self.format_handler {
            Some(h) => h.serialize(value)?,
            None => value.to_string(),
        };
        self.io_handler
            .write(&serialized_value, self.path.as_ref().map(String::as_str))
    }

    pub fn parse(&self, content: &str) -> Result<Value> {
        match &self.format_handler {
            Some(h) => h.parse(content),
            // An idea for the future: envioromnent IO handlers which are not associated with
            // any file format. They will have to skip the format parsing
            None => Err(anyhow!(
                "Inputs/outouts witouth defined formats are not supported"
            )),
        }
    }
}
