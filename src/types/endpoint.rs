use anyhow::{bail, Result};
use serde_json::Value;

use crate::{
    format_handlers::{self, FormatHandler},
    io::{self, IoHandler},
    types::format::Format,
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
    pub fn parse(s: &str, is_input: bool) -> Result<Self> {
        let (fmt, io_scheme, path) = if let Some(uri) = Uri::try_or_none_from_string(s) {
            let fmt = if let Some(f) = Format::from_scheme(&uri.scheme) {
                f
            } else if uri.scheme == "file" || uri.scheme.starts_with("stdin") || uri.scheme.starts_with("stdout") {
                Format::try_detect_format_from_path(&uri.path)?
            } else {
                bail!("Unknown endpoint scheme: {}", uri.scheme);
            };

            // Derive the base IO scheme (strip format suffix, e.g. "stdin-yaml" -> "stdin")
            let io_scheme = if let Some(dash) = uri.scheme.rfind('-') {
                if Format::from_scheme(&uri.scheme).is_some() {
                    uri.scheme[..dash].to_string()
                } else {
                    uri.scheme.clone()
                }
            } else {
                uri.scheme.clone()
            };

            (fmt, io_scheme, uri.path)
        } else {
            let fmt = Format::try_detect_format_from_path(s)?;
            let io_scheme = if is_input { "file" } else { "stdout" }.to_string();
            (fmt, io_scheme, s.to_string())
        };

        Ok(Self {
            io: io::get_handler(&io_scheme)?,
            format: format_handlers::get_handler(fmt),
            path,
        })
    }

    /// Reads raw string content from this endpoint (for Jinja rendering before parsing).
    pub fn read_raw(&self) -> Result<String> {
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
