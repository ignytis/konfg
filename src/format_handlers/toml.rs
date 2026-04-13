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

    fn supports(&self, scheme: &str) -> bool {
        scheme.ends_with("-toml")
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
        assert!(handler.supports("file-toml"));
        assert!(handler.supports("stdio-toml"));
        assert!(!handler.supports("file-json"));
    }
}
