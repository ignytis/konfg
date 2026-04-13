use anyhow::Result;
use serde_json::Value;

use crate::format_handlers::FormatHandler;
use crate::utils::hashmap::{hashmap_flatten, hashmap_new_from_flat_hashmap};

/// A handler for managing Java properties configuration files.
pub struct PropertiesHandler;

impl FormatHandler for PropertiesHandler {
    fn parse(&self, content: &str) -> Result<Value> {
        let props = java_properties::read(content.as_bytes())?;
        Ok(hashmap_new_from_flat_hashmap(props, "."))
    }

    fn serialize(&self, value: &Value) -> Result<String> {
        let mut map = std::collections::HashMap::new();
        hashmap_flatten(value, String::new(), &mut map, ".", false);
        let mut buf = Vec::new();
        java_properties::write(&mut buf, &map)?;
        Ok(String::from_utf8(buf)?)
    }

    fn supports(&self, scheme: &str) -> bool {
        scheme.ends_with("-properties")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_properties_parse() {
        let handler = PropertiesHandler;
        let content = "a.b.c=val1\na.b.d=val2\nx.y=val3";
        let parsed = handler.parse(content).unwrap();
        assert_eq!(
            parsed,
            json!({
                "a": {
                    "b": {
                        "c": "val1",
                        "d": "val2"
                    }
                },
                "x": {
                    "y": "val3"
                }
            })
        );
    }

    #[test]
    fn test_properties_serialize() {
        let handler = PropertiesHandler;
        let value = json!({
            "a": {
                "b": "c"
            }
        });
        let serialized = handler.serialize(&value).unwrap();
        assert!(serialized.contains("a.b=c"));
    }

    #[test]
    fn test_properties_supports() {
        let handler = PropertiesHandler;
        assert!(handler.supports("file-properties"));
        assert!(handler.supports("stdio-properties"));
        assert!(!handler.supports("file-json"));
    }
}
