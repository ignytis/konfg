use anyhow::Result;
use serde_json::Value;

use crate::format_handlers::FormatHandler;
use crate::utils::hashmap::hashmap_new_from_flat_hashmap;

/// A handler for managing Java properties configuration files.
pub struct PropertiesHandler;

impl FormatHandler for PropertiesHandler {
    fn parse(&self, content: &str) -> Result<Value> {
        let props = java_properties::read(content.as_bytes())?;
        Ok(hashmap_new_from_flat_hashmap(props, "."))
    }

    fn serialize(&self, value: &Value) -> Result<String> {
        let mut map = std::collections::HashMap::new();
        flatten(value, String::new(), &mut map, ".", false);
        let mut buf = Vec::new();
        java_properties::write(&mut buf, &map)?;
        Ok(String::from_utf8(buf)?)
    }
}

/// Flattens a nested `Value` into a flat `HashMap<String, String>` using the provided delimiter.
pub fn flatten(
    value: &Value,
    prefix: String,
    map: &mut std::collections::HashMap<String, String>,
    delimiter: &str,
    uppercase: bool,
) {
    match value {
        Value::Object(obj) => {
            for (k, v) in obj {
                let key = if uppercase { k.to_uppercase() } else { k.clone() };
                let new_prefix = if prefix.is_empty() {
                    key
                } else {
                    format!("{}{}{}", prefix, delimiter, key)
                };
                flatten(v, new_prefix, map, delimiter, uppercase);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let new_prefix = if prefix.is_empty() {
                    i.to_string()
                } else {
                    format!("{}{}{}", prefix, delimiter, i)
                };
                flatten(v, new_prefix, map, delimiter, uppercase);
            }
        }
        Value::String(s) => {
            map.insert(prefix, s.clone());
        }
        Value::Bool(b) => {
            map.insert(prefix, b.to_string());
        }
        Value::Number(n) => {
            map.insert(prefix, n.to_string());
        }
        Value::Null => {
            map.insert(prefix, String::new());
        }
    }
}
