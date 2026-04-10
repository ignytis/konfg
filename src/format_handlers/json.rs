use anyhow::Result;
use serde_json::Value;

use crate::format_handlers::FormatHandler;

/// A handler for managing JSON configuration files.
pub struct JsonHandler;

impl FormatHandler for JsonHandler {
    fn parse(&self, content: &str) -> Result<Value> {
        Ok(serde_json::from_str(content)?)
    }

    fn serialize(&self, value: &Value) -> Result<String> {
        Ok(serde_json::to_string_pretty(value)?)
    }


    fn supports(&self, scheme: &str) -> bool {
        scheme.ends_with("-json")
    }
}
