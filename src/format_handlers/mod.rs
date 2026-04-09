mod dotenv;
mod json;
mod properties;
mod toml;
mod yaml;

use anyhow::Result;
use serde_json::Value;

use crate::types::format::Format;

/// A trait for defining how to parse and serialize configuration formats.
pub trait FormatHandler {
    fn parse(&self, content: &str) -> Result<Value>;
    fn serialize(&self, value: &Value) -> Result<String>;
}

/// Retrieves the appropriate handler for a given `Format`.
pub fn get_handler(format: Format) -> Box<dyn FormatHandler> {
    match format {
        Format::Yaml => Box::new(yaml::YamlHandler),
        Format::Json => Box::new(json::JsonHandler),
        Format::Toml => Box::new(toml::TomlHandler),
        Format::Properties => Box::new(properties::PropertiesHandler),
        Format::Dotenv => Box::new(dotenv::DotenvHandler),
    }
}
