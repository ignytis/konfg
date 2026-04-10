mod dotenv;
mod json;
mod properties;
mod toml;
mod yaml;

use anyhow::{anyhow, Result};
use serde_json::Value;

/// A trait for defining how to parse and serialize configuration formats.
pub trait FormatHandler {
    fn parse(&self, content: &str) -> Result<Value>;
    fn serialize(&self, value: &Value) -> Result<String>;
    /// Checks if this handler supports the given scheme
    /// For example: "stdio-yaml", "file-toml"
    fn supports(&self, scheme: &str) -> bool;
}

/// Factory method to get the appropriate firnat handler for the given scheme
/// Iterates over all registered handlers and returns the first one that supports the kind.
pub fn get_handler(scheme: &str) -> Result<Box<dyn FormatHandler>> {
    let handlers: Vec<Box<dyn FormatHandler>> = vec![
        Box::new(dotenv::DotenvHandler),
        Box::new(json::JsonHandler),
        Box::new(properties::PropertiesHandler),
        Box::new(toml::TomlHandler),
        Box::new(yaml::YamlHandler),
    ];

    for handler in handlers {
        if handler.supports(scheme) {
            return Ok(handler);
        }
    }

    Err(anyhow!("No IO handler found for: {}", scheme))
}
