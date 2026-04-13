use anyhow::Result;
use serde_json::Value;

use crate::{
    format_handlers::FormatHandler,
    utils::hashmap::{hashmap_flatten, hashmap_new_from_flat_hashmap},
};

/// A handler for managing `.env` configuration files.
pub struct DotenvHandler;

impl FormatHandler for DotenvHandler {
    fn parse(&self, content: &str) -> Result<Value> {
        let iter = dotenvy::Iter::new(content.as_bytes());
        let mut props = std::collections::HashMap::new();
        for item in iter {
            let (key, val) = item?;
            props.insert(key.to_lowercase(), val);
        }
        Ok(hashmap_new_from_flat_hashmap(props, "__"))
    }

    fn serialize(&self, value: &Value) -> Result<String> {
        let map = hashmap_flatten(value, "", "__", true);
        let mut res = String::new();
        for (k, v) in map {
            res.push_str(&format!("{}={}\n", k, v));
        }
        Ok(res)
    }

    fn supports(&self, scheme: &str) -> bool {
        scheme.ends_with("-dotenv")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_dotenv_parse() {
        let handler = DotenvHandler;
        let content = "APP__DB__HOST=localhost\nAPP__DB__PORT=5432";
        let parsed = handler.parse(content).unwrap();
        assert_eq!(
            parsed,
            json!({
                "app": {
                    "db": {
                        "host": "localhost",
                        "port": "5432"
                    }
                }
            })
        );
    }

    #[test]
    fn test_dotenv_serialize() {
        let handler = DotenvHandler;
        let value = json!({
            "app": {
                "db": {
                    "host": "localhost"
                }
            }
        });
        let serialized = handler.serialize(&value).unwrap();
        assert!(serialized.contains("APP__DB__HOST=localhost"));
    }

    #[test]
    fn test_dotenv_supports() {
        let handler = DotenvHandler;
        assert!(handler.supports("file-dotenv"));
        assert!(handler.supports("stdio-dotenv"));
        assert!(!handler.supports("file-json"));
    }
}
