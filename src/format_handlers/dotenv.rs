use anyhow::Result;
use serde_json::Value;

use crate::format_handlers::{properties::flatten, FormatHandler};
use crate::utils::hashmap::hashmap_new_from_flat_hashmap;

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
        let mut map = std::collections::HashMap::new();
        flatten(value, String::new(), &mut map, "__", true);
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
