use anyhow::Result;
use serde_json::Value;

use crate::format_handlers::FormatHandler;

/// A handler for managing JSON configuration files.
#[derive(Clone)]
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

    fn clone_box(&self) -> Box<dyn FormatHandler> {
        Box::new(self.clone())
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
        let handler = JsonHandler;
        assert!(handler.supports("file-json"));
        assert!(handler.supports("stdio-json"));
        assert!(!handler.supports("file-yaml"));
    }
}
