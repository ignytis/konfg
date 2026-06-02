use anyhow::Result;
use serde_json::Value;

use crate::file_format_handlers::FormatHandler;

pub const KIND: &str = "json";
pub const EXTENSIONS: &[&str] = &["json"];

/// A handler for managing JSON configuration files.
#[derive(Clone)]
pub struct JsonHandler;

impl JsonHandler {
    pub fn create() -> Box<dyn FormatHandler> {
        Box::new(JsonHandler)
    }
}

impl FormatHandler for JsonHandler {
    fn parse(&self, content: &str) -> Result<Value> {
        Ok(serde_json::from_str(content)?)
    }

    fn serialize(&self, value: &Value) -> Result<String> {
        Ok(serde_json::to_string_pretty(value)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_parse() {
        let handler = JsonHandler;
        let content = r#"{"key": "value", "nested": {"a": 1}}"#;
        let parsed = handler.parse(content).unwrap();
        assert_eq!(parsed, json!({"key": "value", "nested": {"a": 1}}));
    }

    #[test]
    fn test_json_serialize() {
        let handler = JsonHandler;
        let value = json!({"key": "value"});
        let serialized = handler.serialize(&value).unwrap();
        assert!(serialized.contains(r#""key": "value""#));
    }

    #[test]
    fn test_json_supports() {
        assert_eq!(KIND, "json");
    }
}
