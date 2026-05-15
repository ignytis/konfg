//! Utilities for configuration values

use anyhow::{bail, Result};
use serde_json::Value;

/// Deep-merge `src` into `dst`. Later values overwrite earlier ones for scalars.
/// Merging a collection with a scalar (or vice-versa) is an error.
pub fn cfg_values_deep_merge(dst: &mut Value, src: Value) -> Result<()> {
    match (dst, src) {
        (Value::Object(d), Value::Object(s)) => {
            for (k, sv) in s {
                let dv = d.entry(k).or_insert(Value::Null);
                if dv.is_null() {
                    *dv = sv;
                } else {
                    cfg_values_deep_merge(dv, sv)?;
                }
            }
        }
        (Value::Array(d), Value::Array(s)) => {
            // Replace array wholesale (last wins)
            *d = s.into_iter().collect();
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
            *dst = src;
        }
    }
    Ok(())
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

        cfg_values_deep_merge(&mut dst, src)?;

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
    fn test_cfg_values_deep_merge_overwrite() -> Result<()> {
        let mut dst = json!({
            "key1": "val1",
            "key2": "val2"
        });
        let src = json!({
            "key1": "new_val1"
        });

        cfg_values_deep_merge(&mut dst, src)?;

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

        cfg_values_deep_merge(&mut dst, src)?;

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
    fn test_cfg_values_deep_merge_array() -> Result<()> {
        let mut dst = json!(["a", "b"]);
        let src = json!(["c", "d"]);

        cfg_values_deep_merge(&mut dst, src)?;

        assert_eq!(dst, json!(["c", "d"]));

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

        assert!(cfg_values_deep_merge(&mut dst, src).is_err());

        Ok(())
    }
}
