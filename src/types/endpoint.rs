use anyhow::Result;
use serde_json::Value;

use crate::{
    format_handlers::{self, FormatHandler},
    io::{self, IoHandler},
    utils::uri::Uri
};

/// Represents a configuration source or destination with associated IO and format handlers.
pub struct Endpoint {
    io: Box<dyn IoHandler>,
    format: Box<dyn FormatHandler>,
    path: String,
}

impl Endpoint {
    /// Parses a URI string into an `Endpoint`.
    /// `is_input` controls the default scheme when no URI scheme is present.
    pub fn parse(uri_str: &str, is_input: bool) -> Result<Self> {
        let uri = Uri::try_or_default_from_string(uri_str, is_input);
        Ok(Self {
            io: io::get_handler(&uri.scheme)?,
            format: format_handlers::get_handler(&uri.scheme)?,
            path: uri.path,
        })
    }

    /// Reads raw string content from this endpoint (for Jinja rendering before parsing).
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
