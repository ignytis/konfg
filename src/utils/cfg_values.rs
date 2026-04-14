//! Utilities for configuration values

use anyhow::{Result, bail};
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
