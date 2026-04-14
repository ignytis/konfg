use anyhow::Result;
use serde_json::Value;

use crate::handlers::format::FormatHandler;
use crate::utils::hashmap::{hashmap_flatten, hashmap_new_from_flat_hashmap};

/// A handler for managing Java properties configuration files.
#[derive(Clone)]
pub struct PropertiesHandler;

impl FormatHandler for PropertiesHandler {
    fn parse(&self, content: &str) -> Result<Value> {
        let props = java_properties::read(content.as_bytes())?;
        Ok(hashmap_new_from_flat_hashmap(props, "."))
    }

    fn serialize(&self, value: &Value) -> Result<String> {
        let map = hashmap_flatten(value, "", ".", false);
        let mut buf = Vec::new();
        java_properties::write(&mut buf, &map)?;
        Ok(String::from_utf8(buf)?)
    }

    fn supports(&self, format: &str) -> bool {
        format == "properties"
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
        assert!(handler.supports("properties"));
        assert!(!handler.supports("json"));
    }
}
