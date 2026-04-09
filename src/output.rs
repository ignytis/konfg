use anyhow::{bail, Result};
use serde_json::Value;

use crate::format_handlers::get_handler;
use crate::io;
use crate::types::format::Format;
use crate::utils::uri::Uri;

/// Represents the target for configuration output.
pub struct OutputTarget {
    pub format: Format,
    pub dest: String,
}

impl OutputTarget {
    pub fn parse(s: &str) -> Result<Self> {
        if let Some(uri) = Uri::try_or_none_from_string(s) {
            let scheme = uri.scheme.clone();

            if let Some(fmt) = Format::from_scheme(&scheme) {
                return Ok(Self {
                    format: fmt,
                    dest: uri.path.to_string(),
                });
            }

            if scheme == "file" {
                let fmt = Format::try_detect_format_from_path(&uri.path)?;
                return Ok(Self {
                    format: fmt,
                    dest: uri.path,
                });
            }

            bail!("Unknown output scheme: {}", &scheme);
        }

        let fmt = Format::try_detect_format_from_path(s)?;
        Ok(Self {
            format: fmt,
            dest: s.to_string(),
        })
    }

    pub fn write(&self, value: &Value) -> Result<()> {
        let uri = Uri::try_or_default_from_string(&self.dest, false);
        let handler = io::get_handler(&uri.scheme)?;
        let serialized = get_handler(self.format).serialize(value)?;
        handler.write(&uri.path, &serialized)
    }
}

