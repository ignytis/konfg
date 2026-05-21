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
        let parts = hashmap_parse_key_parts(key);
        map = hashmap_insert_nested_value(map, &parts, Value::String(val.to_string()));
    }
    Ok(map)
}

/// Parses a key into parts, considering dots as separators and double dots as escaped dots.
pub fn hashmap_parse_key_parts(key: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = key.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '.' {
            if chars.peek() == Some(&'.') {
                // Escaped dot: ".." -> "."
                chars.next();
                current.push('.');
            } else {
                // Separator: "."
                parts.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    parts.push(current);
    parts
}

/// Inserts a `value` into `map` using `parts` as a path.
pub fn hashmap_insert_nested_value(
    mut map: serde_json::Map<String, Value>,
    parts: &[String],
    val: Value,
) -> serde_json::Map<String, Value> {
    if parts.is_empty() {
        return map;
    }
    if parts.len() == 1 {
        map.insert(parts[0].clone(), val);
        return map;
    }

    let head = &parts[0];
    let tail = &parts[1..];

    let inner = match map.remove(head) {
        Some(Value::Object(inner_map)) => inner_map,
        _ => serde_json::Map::new(),
    };
    map.insert(
        head.clone(),
        Value::Object(hashmap_insert_nested_value(inner, tail, val)),
    );
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

    #[test]
    fn test_hashmap_new_from_kv_params_simple() -> Result<()> {
        let params = vec!["key1=val1".to_string(), "key2=val2".to_string()];

        let result = hashmap_new_from_kv_params(&params)?;
        assert_eq!(
            json!(result),
            json!({
                "key1": "val1",
                "key2": "val2"
            })
        );

        Ok(())
    }

    #[test]
    fn test_hashmap_new_from_kv_params_nested() -> Result<()> {
        let params = vec![
            "a.b.c=val1".to_string(),
            "a.b.d=val2".to_string(),
            "x.y=val3".to_string(),
        ];

        let result = hashmap_new_from_kv_params(&params)?;
        assert_eq!(
            json!(result),
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

        Ok(())
    }

    #[test]
    fn test_hashmap_parse_key_parts() {
        assert_eq!(hashmap_parse_key_parts("a.b.c"), vec!["a", "b", "c"]);
        assert_eq!(hashmap_parse_key_parts("a..b.c"), vec!["a.b", "c"]);
        assert_eq!(hashmap_parse_key_parts("a....b"), vec!["a..b"]);
        assert_eq!(hashmap_parse_key_parts("a...b"), vec!["a.", "b"]);
    }

    #[test]
    fn test_hashmap_new_from_kv_params_escaped() -> Result<()> {
        let params = vec!["a..b.c=val1".to_string()];

        let result = hashmap_new_from_kv_params(&params)?;
        assert_eq!(
            json!(result),
            json!({
                "a.b": {
                    "c": "val1"
                }
            })
        );

        Ok(())
    }

    #[test]
    fn test_hashmap_flatten_simple() {
        let value = json!({
            "key1": "val1",
            "key2": "val2"
        });

        let result = hashmap_flatten(&value, "", ".", false);
        assert_eq!(
            result,
            HashMap::from([
                ("key1".to_string(), "val1".to_string()),
                ("key2".to_string(), "val2".to_string())
            ])
        );
    }

    #[test]
    fn test_hashmap_flatten_nested() {
        let value = json!({
            "a": {
                "b": {
                    "c": "val1",
                    "d": "val2"
                }
            },
            "x": {
                "y": "val3"
            }
        });

        let result = hashmap_flatten(&value, "", ".", false);
        assert_eq!(
            result,
            HashMap::from([
                ("a.b.c".to_string(), "val1".to_string()),
                ("a.b.d".to_string(), "val2".to_string()),
                ("x.y".to_string(), "val3".to_string())
            ])
        );
    }

    #[test]
    fn test_hashmap_flatten_uppercase() {
        let value = json!({
            "key1": "val1",
            "key2": "val2"
        });

        let result = hashmap_flatten(&value, "", ".", true);
        assert_eq!(
            result,
            HashMap::from([
                ("KEY1".to_string(), "val1".to_string()),
                ("KEY2".to_string(), "val2".to_string())
            ])
        );
    }
}
