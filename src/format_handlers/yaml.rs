use anyhow::Result;
use serde_json::Value;

use crate::format_handlers::FormatHandler;

/// A handler for managing YAML configuration files.
#[derive(Clone)]
pub struct YamlHandler;

impl FormatHandler for YamlHandler {
    fn parse(&self, content: &str) -> Result<Value> {
        Ok(serde_yaml::from_str(content)?)
    }

    fn serialize(&self, value: &Value) -> Result<String> {
        Ok(serde_yaml::to_string(value)?)
    }

    fn supports(&self, scheme: &str) -> bool {
        scheme.ends_with("-yaml")
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
    fn test_yaml_parse() {
        let handler = YamlHandler;
        let content = "key: value\nnested:\n  a: 1";
        let parsed = handler.parse(content).unwrap();
        assert_eq!(parsed, json!({"key": "value", "nested": {"a": 1}}));
    }

    #[test]
    fn test_yaml_serialize() {
        let handler = YamlHandler;
        let value = json!({"key": "value"});
        let serialized = handler.serialize(&value).unwrap();
        assert!(serialized.contains("key: value"));
    }

    #[test]
    fn test_yaml_supports() {
        let handler = YamlHandler;
        assert!(handler.supports("file-yaml"));
        assert!(handler.supports("stdio-yaml"));
        assert!(!handler.supports("file-json"));
    }
}
