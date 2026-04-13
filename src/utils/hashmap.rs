use anyhow::Result;
use serde_json::Value;

/// Creates a JSON value object from a flat hashmap.
///
/// NB! Do NOT use it for nested hashmaps because this function is not recursive.
/// TODO: replace it with a smarter function which uses recursion.
pub fn hashmap_new_from_flat_hashmap(
    props: std::collections::HashMap<String, String>,
    delimiter: &str,
) -> Value {
    let mut root = serde_json::Map::new();
    for (key, val) in props {
        let parts: Vec<&str> = key.split(delimiter).collect();
        let mut current = &mut root;
        for i in 0..parts.len() - 1 {
            let part = parts[i];
            let entry = current
                .entry(part.to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            if !entry.is_object() {
                *entry = Value::Object(serde_json::Map::new());
            }
            current = entry.as_object_mut().unwrap();
        }
        current.insert(parts.last().unwrap().to_string(), Value::String(val));
    }
    Value::Object(root)
}

/// Creates a nested hashmap from key-value pairs like ['key1=val1', 'key2=val2']
pub fn hashmap_new_from_kv_params(params: &[String]) -> Result<serde_json::Map<String, Value>> {
    let mut map: serde_json::Map<String, Value> = serde_json::Map::new();
    for p in params {
        let (key, val) = p
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("Invalid param '{p}': expected key=value"))?;
        map = hashmap_insert_value_dot_separator(map, key, Value::String(val.to_string()));
    }
    Ok(map)
}

/// Inserts a `value` into `map` using `key`.
/// Considers the nested structure, spltting key by dots.
fn hashmap_insert_value_dot_separator(
    mut map: serde_json::Map<String, Value>,
    key: &str,
    val: Value,
) -> serde_json::Map<String, Value> {
    if let Some((head, tail)) = key.split_once('.') {
        let inner = match map.remove(head) {
            Some(Value::Object(inner_map)) => inner_map,
            _ => serde_json::Map::new(),
        };
        map.insert(
            head.to_string(),
            Value::Object(hashmap_insert_value_dot_separator(inner, tail, val)),
        );
    } else {
        map.insert(key.to_string(), val);
    }
    map
}

/// Flattens a nested `Value` into a flat `HashMap<String, String>` using the provided delimiter.
pub fn hashmap_flatten(
    value: &Value,
    prefix: &str,
    delimiter: &str,
    uppercase: bool,
) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    match value {
        Value::Object(obj) => {
            for (k, v) in obj {
                let key = if uppercase {
                    k.to_uppercase()
                } else {
                    k.clone()
                };
                let new_prefix = if prefix.is_empty() {
                    key
                } else {
                    format!("{}{}{}", prefix, delimiter, key)
                };
                map.extend(hashmap_flatten(v, &new_prefix, delimiter, uppercase));
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let new_prefix = if prefix.is_empty() {
                    i.to_string()
                } else {
                    format!("{}{}{}", prefix, delimiter, i)
                };
                map.extend(hashmap_flatten(v, &new_prefix, delimiter, uppercase));
            }
        }
        Value::String(s) => {
            map.insert(prefix.to_string(), s.clone());
        }
        Value::Bool(b) => {
            map.insert(prefix.to_string(), b.to_string());
        }
        Value::Number(n) => {
            map.insert(prefix.to_string(), n.to_string());
        }
        Value::Null => {
            map.insert(prefix.to_string(), String::new());
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_hashmap_new_from_flat_hashmap_simple() {
        let mut props = HashMap::new();
        props.insert("key1".to_string(), "val1".to_string());
        props.insert("key2".to_string(), "val2".to_string());

        let result = hashmap_new_from_flat_hashmap(props, ".");
        assert_eq!(
            result,
            json!({
                "key1": "val1",
                "key2": "val2"
            })
        );
    }

    #[test]
    fn test_hashmap_new_from_flat_hashmap_nested() {
        let mut props = HashMap::new();
        props.insert("a.b.c".to_string(), "val1".to_string());
        props.insert("a.b.d".to_string(), "val2".to_string());
        props.insert("x.y".to_string(), "val3".to_string());

        let result = hashmap_new_from_flat_hashmap(props, ".");
        assert_eq!(
            result,
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
}
