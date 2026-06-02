pub mod dotenv;
mod json;
mod properties;
mod toml;
mod yaml;

use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use serde_json::Value;

pub type FormatHandlerCreatorFn = fn() -> Box<dyn FormatHandler>;

pub const REGISTERED_HANDLERS: LazyLock<
    Vec<(
        &'static str,
        &'static [&'static str],
        FormatHandlerCreatorFn,
    )>,
> = LazyLock::new(|| {
    vec![
        (
            dotenv::KIND,
            dotenv::EXTENSIONS,
            dotenv::DotenvHandler::create,
        ),
        (json::KIND, json::EXTENSIONS, json::JsonHandler::create),
        (
            properties::KIND,
            properties::EXTENSIONS,
            properties::PropertiesHandler::create,
        ),
        (toml::KIND, toml::EXTENSIONS, toml::TomlHandler::create),
        (yaml::KIND, yaml::EXTENSIONS, yaml::YamlHandler::create),
    ]
});

/// A trait for defining how to parse and serialize configuration formats.
pub trait FormatHandler: Send + Sync {
    fn parse(&self, content: &str) -> Result<Value>;
    fn serialize(&self, value: &Value) -> Result<String>;
}

/// Factory method to get the appropriate format handler for the given format name.
/// Iterates over all registered handlers and returns the first one that supports the format.
pub fn get_handler_for_format(format: &str) -> Option<Box<dyn FormatHandler>> {
    for (id, _, creator) in REGISTERED_HANDLERS.iter() {
        if id.eq(&format) {
            return Some(creator());
        }
    }

    None
}

/// Factory method to get the appropriate IO handler for the given file extension.
pub fn get_handler_for_file_extension(
    extension: &str,
) -> Result<(&'static str, Box<dyn FormatHandler>)> {
    for (id, extensions, creator) in REGISTERED_HANDLERS.iter() {
        if extensions.contains(&extension) {
            return Ok((*id, creator()));
        }
    }

    Err(anyhow!("No IO handler found for extension: {}", extension))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_handler_json() {
        let handler = get_handler_for_format("json");
        assert!(handler.is_some(), "JSON handler should be registered");
    }

    #[test]
    fn test_get_handler_yaml() {
        let handler = get_handler_for_format("yaml");
        assert!(handler.is_some(), "YAML handler should be registered");
    }

    #[test]
    fn test_get_handler_toml() {
        let handler = get_handler_for_format("toml");
        assert!(handler.is_some(), "TOML handler should be registered");
    }

    #[test]
    fn test_get_handler_properties() {
        let handler = get_handler_for_format("properties");
        assert!(handler.is_some(), "Properties handler should be registered");
    }

    #[test]
    fn test_get_handler_dotenv() {
        let handler = get_handler_for_format("dotenv");
        assert!(handler.is_some(), "Dotenv handler should be registered");
    }

    #[test]
    fn test_get_handler_unknown() {
        let handler = get_handler_for_format("unknown");
        assert!(
            handler.is_none(),
            "Unknown format should not have a handler"
        );
    }
}
