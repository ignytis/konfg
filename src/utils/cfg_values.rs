//! Utilities for configuration values

use std::collections::{HashMap, VecDeque};

use anyhow::{Result, anyhow, bail};
use serde_json::Value;

/// Deep-merge `src` into `dst` using provided strategies.
/// Merging a collection with a scalar (or vice-versa) is an error, unless strategy is 'overwrite'.
pub fn cfg_values_deep_merge(
    dst: &mut Value,
    src: &Value,
    strategies: &HashMap<String, VecDeque<String>>,
) -> Result<()> {
    let strategy_args = strategies.get("");
    let strategy = strategy_args
        .and_then(|v| v.front())
        .map(|s| s.as_str())
        .unwrap_or("simple");

    if strategy == "overwrite" {
        *dst = src.clone();
        return Ok(());
    }

    match (dst, src) {
        (Value::Object(d), Value::Object(s)) => {
            for (k, sv) in s {
                let dv = d.entry(k).or_insert(Value::Null);
                let next_strategies = advance_strategies(strategies, k);
                if dv.is_null() {
                    *dv = sv.clone();
                } else {
                    cfg_values_deep_merge(dv, sv, &next_strategies)?;
                }
            }
        }
        (Value::Array(d), Value::Array(s)) => {
            if strategy == "merge_by_key" {
                let key_name = strategy_args
                    .and_then(|v| v.get(1))
                    .ok_or_else(|| anyhow!("merge_by_key requires a key argument"))?;

                let next_strategies = advance_strategies(strategies, "[]");

                for sv in s {
                    if let Some(s_key_val) = sv.get(key_name) {
                        let mut found = false;
                        for dv in d.iter_mut() {
                            if dv.get(key_name) == Some(s_key_val) {
                                cfg_values_deep_merge(dv, sv, &next_strategies)?;
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            d.push(sv.clone());
                        }
                    } else {
                        d.push(sv.clone());
                    }
                }
            } else {
                // simple merge for arrays: append
                for sv in s {
                    d.push(sv.clone());
                }
            }
        }
        (dst, src) => {
            // Disallow overwriting a collection with a scalar or vice-versa
            let dst_is_coll = dst.is_object() || dst.is_array();
            let src_is_coll = src.is_object() || src.is_array();
            if dst_is_coll != src_is_coll {
                bail!(
                    "Type mismatch during merge: cannot overwrite {} with {}",
                    if dst_is_coll { "collection" } else { "scalar" },
                    if src_is_coll { "collection" } else { "scalar" }
                );
            }
            *dst = src.clone();
        }
    }
    Ok(())
}

/// Advance strategies map by one level.
fn advance_strategies(
    strategies: &HashMap<String, VecDeque<String>>,
    segment: &str,
) -> HashMap<String, VecDeque<String>> {
    let mut next = HashMap::new();
    let prefix = format!("{}.", segment);
    for (path, strategy) in strategies {
        if path.starts_with(&prefix) {
            next.insert(path[prefix.len()..].to_string(), strategy.clone());
        } else if path == segment {
            next.insert("".to_string(), strategy.clone());
        }
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cfg_values_deep_merge_simple() -> Result<()> {
        let mut dst = json!({
            "key1": "val1",
            "key2": "val2"
        });
        let src = json!({
            "key3": "val3"
        });

        cfg_values_deep_merge(&mut dst, &src, &HashMap::new())?;

        assert_eq!(
            dst,
            json!({
                "key1": "val1",
                "key2": "val2",
                "key3": "val3"
            })
        );

        Ok(())
    }

    #[test]
    fn test_cfg_values_deep_merge_overwrite_val() -> Result<()> {
        let mut dst = json!({
            "key1": "val1",
            "key2": "val2"
        });
        let src = json!({
            "key1": "new_val1"
        });

        cfg_values_deep_merge(&mut dst, &src, &HashMap::new())?;

        assert_eq!(
            dst,
            json!({
                "key1": "new_val1",
                "key2": "val2"
            })
        );

        Ok(())
    }

    #[test]
    fn test_cfg_values_deep_merge_nested() -> Result<()> {
        let mut dst = json!({
            "key1": {
                "subkey1": "val1"
            }
        });
        let src = json!({
            "key1": {
                "subkey2": "val2"
            }
        });

        cfg_values_deep_merge(&mut dst, &src, &HashMap::new())?;

        assert_eq!(
            dst,
            json!({
                "key1": {
                    "subkey1": "val1",
                    "subkey2": "val2"
                }
            })
        );

        Ok(())
    }

    #[test]
    fn test_cfg_values_deep_merge_array_append() -> Result<()> {
        let mut dst = json!(["a", "b"]);
        let src = json!(["c", "d"]);

        cfg_values_deep_merge(&mut dst, &src, &HashMap::new())?;

        assert_eq!(dst, json!(["a", "b", "c", "d"]));

        Ok(())
    }

    #[test]
    fn test_cfg_values_deep_merge_type_mismatch() -> Result<()> {
        let mut dst = json!({
            "key1": "val1"
        });
        let src = json!({
            "key1": ["a", "b"]
        });

        assert!(cfg_values_deep_merge(&mut dst, &src, &HashMap::new()).is_err());

        Ok(())
    }

    #[test]
    fn test_cfg_values_deep_merge_strategy_overwrite() -> Result<()> {
        let mut dst = json!({
            "a": "a1",
            "e": {
                "aa": "a1",
                "bb": "b1"
            }
        });
        let src = json!({
            "a": "a2",
            "e": {
                "bb": "b2",
                "cc": "c2"
            }
        });

        let mut strategies = HashMap::new();
        strategies.insert("e".to_string(), VecDeque::from(["overwrite".to_string()]));

        cfg_values_deep_merge(&mut dst, &src, &strategies)?;

        assert_eq!(
            dst,
            json!({
                "a": "a2",
                "e": {
                    "bb": "b2",
                    "cc": "c2"
                }
            })
        );

        Ok(())
    }

    #[test]
    fn test_cfg_values_deep_merge_strategy_merge_by_key() -> Result<()> {
        let mut dst = json!({
            "mydata": [
                { "name": "a", "value": "a1" },
                { "name": "b", "value": "b1", "test": "bb1" }
            ]
        });
        let src = json!({
            "mydata": [
                { "name": "b", "value": "b2" },
                { "name": "c", "value": "c1" }
            ]
        });

        let mut strategies = HashMap::new();
        strategies.insert(
            "mydata".to_string(),
            VecDeque::from(["merge_by_key".to_string(), "name".to_string()]),
        );

        cfg_values_deep_merge(&mut dst, &src, &strategies)?;

        assert_eq!(
            dst,
            json!({
                "mydata": [
                    { "name": "a", "value": "a1" },
                    { "name": "b", "value": "b2", "test": "bb1" },
                    { "name": "c", "value": "c1" }
                ]
            })
        );

        Ok(())
    }

    #[test]
    fn test_cfg_values_deep_merge_strategy_nested_paths() -> Result<()> {
        let mut dst = json!({
            "levela": {
                "levelb": [ { "id": 1, "val": "v1" } ]
            }
        });
        let src = json!({
            "levela": {
                "levelb": [ { "id": 1, "val": "v2" }, { "id": 2, "val": "v3" } ]
            }
        });

        let mut strategies = HashMap::new();
        strategies.insert(
            "levela.levelb".to_string(),
            VecDeque::from(["merge_by_key".to_string(), "id".to_string()]),
        );

        cfg_values_deep_merge(&mut dst, &src, &strategies)?;

        assert_eq!(
            dst,
            json!({
                "levela": {
                    "levelb": [
                        { "id": 1, "val": "v2" },
                        { "id": 2, "val": "v3" }
                    ]
                }
            })
        );

        Ok(())
    }
}
