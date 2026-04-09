use anyhow::{bail, Result};
use serde_json::Value;
use std::fs;
use std::io::{self, Write};

use crate::format_handlers::get_handler;
use crate::types::format::Format;
use crate::utils::uri::Uri;

/// Represents the target for configuration output.
pub struct OutputTarget {
    pub format: Format,
    pub dest: Dest,
}

/// The destination for the output.
pub enum Dest {
    Stdout,
    File(String),
}

impl OutputTarget {
    pub fn parse(s: &str) -> Result<Self> {
        if let Some(uri) = Uri::try_or_none_from_string(s) {
            let scheme = uri.scheme.clone();
            let path = uri.path.clone();

            if uri.scheme.starts_with("stdout") {
                let fmt = Format::from_scheme(uri.scheme)
                    .ok_or_else(|| anyhow::anyhow!("Unknown output scheme: {0}", &scheme))?;
                return Ok(Self {
                    format: fmt,
                    dest: Dest::Stdout,
                });
            }

            if let Some(fmt) = Format::from_scheme(&uri.scheme) {
                return Ok(Self {
                    format: fmt,
                    dest: Dest::File(uri.path.to_string()),
                });
            }

            if &scheme == "file" {
                let fmt = Format::try_detect_format_from_path(uri.path)?;
                return Ok(Self {
                    format: fmt,
                    dest: Dest::File(path),
                });
            }

            bail!("Unknown output scheme: {0}", &uri.scheme);
        }

        let fmt = Format::try_detect_format_from_path(s)?;
        Ok(Self {
            format: fmt,
            dest: Dest::File(s.to_string()),
        })
    }

    pub fn write(&self, value: &Value) -> Result<()> {
        let serialized = get_handler(self.format).serialize(value)?;
        match &self.dest {
            Dest::Stdout => io::stdout().write_all(serialized.as_bytes())?,
            Dest::File(path) => fs::write(path, serialized)?,
        }
        Ok(())
    }
}
