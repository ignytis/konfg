mod dotenv;
mod json;
mod properties;
mod toml;
mod yaml;

use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use serde_json::Value;

const REGISTERED_HANDLERS: LazyLock<Vec<Box<dyn FormatHandler>>> = LazyLock::new(|| {
    vec![
        Box::new(dotenv::DotenvHandler),
        Box::new(json::JsonHandler),
        Box::new(properties::PropertiesHandler),
        Box::new(toml::TomlHandler),
        Box::new(yaml::YamlHandler),
    ]
});

/// A trait for defining how to parse and serialize configuration formats.
pub trait FormatHandler: Send + Sync {
    fn parse(&self, content: &str) -> Result<Value>;
    fn serialize(&self, value: &Value) -> Result<String>;
    /// Checks if this handler supports the given format name, e.g. "yaml", "json", "toml".
    fn supports(&self, format: &str) -> bool;
    /// Clones the handler into a boxed trait object.
    fn clone_box(&self) -> Box<dyn FormatHandler>;
}

impl Clone for Box<dyn FormatHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Factory method to get the appropriate format handler for the given format name.
/// Iterates over all registered handlers and returns the first one that supports the format.
pub fn get_handler(format: &str) -> Result<Box<dyn FormatHandler>> {
    for handler in REGISTERED_HANDLERS.iter() {
        if handler.supports(format) {
            return Ok(handler.clone());
        }
    }

    Err(anyhow!("No format handler found for: {}", format))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_handler_json() {
        assert!(get_handler("json").is_ok());
    }

    #[test]
    fn test_get_handler_yaml() {
        assert!(get_handler("yaml").is_ok());
    }

    #[test]
    fn test_get_handler_toml() {
        assert!(get_handler("toml").is_ok());
    }

    #[test]
    fn test_get_handler_properties() {
        assert!(get_handler("properties").is_ok());
    }

    #[test]
    fn test_get_handler_dotenv() {
        assert!(get_handler("dotenv").is_ok());
    }

    #[test]
    fn test_get_handler_unknown() {
        assert!(get_handler("unknown").is_err());
    }
}
