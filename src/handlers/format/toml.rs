use anyhow::Result;
use serde_json::Value;

use crate::handlers::format::FormatHandler;

/// A handler for managing TOML configuration files.
#[derive(Clone)]
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

    fn supports(&self, format: &str) -> bool {
        format == "toml"
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
    fn test_toml_parse() {
        let handler = TomlHandler;
        let content = r#"key = "value"
[nested]
a = 1"#;
        let parsed = handler.parse(content).unwrap();
        assert_eq!(parsed, json!({"key": "value", "nested": {"a": 1}}));
    }

    #[test]
    fn test_toml_serialize() {
        let handler = TomlHandler;
        let value = json!({"key": "value"});
        let serialized = handler.serialize(&value).unwrap();
        assert!(serialized.contains(r#"key = "value""#));
    }

    #[test]
    fn test_toml_supports() {
        let handler = TomlHandler;
        assert!(handler.supports("toml"));
        assert!(!handler.supports("json"));
    }
}
