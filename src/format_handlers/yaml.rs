use anyhow::Result;
use serde_json::Value;

use crate::format_handlers::FormatHandler;

/// A handler for managing YAML configuration files.
pub struct YamlHandler;

impl FormatHandler for YamlHandler {
    fn parse(&self, content: &str) -> Result<Value> {
        Ok(serde_yaml::from_str(content)?)
    }

    fn serialize(&self, value: &Value) -> Result<String> {
        Ok(serde_yaml::to_string(value)?)
    }
}
