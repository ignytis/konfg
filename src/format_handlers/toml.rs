use anyhow::Result;
use serde_json::Value;

use crate::format_handlers::FormatHandler;

/// A handler for managing TOML configuration files.
pub struct TomlHandler;

impl FormatHandler for TomlHandler {
    fn parse(&self, content: &str) -> Result<Value> {
        let toml_val: toml::Value = toml::from_str(content)?;
        Ok(serde_json::to_value(toml_val)?)
    }

    fn serialize(&self, value: &Value) -> Result<String> {
        let tv: toml::Value = serde_json::from_value(value.clone())?;
        Ok(toml::to_string_pretty(&tv)?)
    }
}
