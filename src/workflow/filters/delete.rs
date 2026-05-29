use std::collections::{HashMap, VecDeque};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    jinja::JinjaEngine,
    utils::hashmap::{hashmap_delete_nested_value, hashmap_parse_key_parts},
    workflow::filters::{BaseFilter, Filter, TryParseFilterResult},
    workflow::stage::{Stage, StageExecutionContext, StageKind},
};

const KIND: &str = "delete";

/// Filter that deletes a parameter from the merged configuration.
#[derive(Clone)]
pub struct DeleteFilter;

impl BaseFilter for DeleteFilter {
    fn supports(&self, kind: &str) -> bool {
        kind == KIND
    }

    fn clone_box(&self) -> Box<dyn BaseFilter> {
        Box::new(self.clone())
    }

    fn try_parse_args(
        &self,
        tokens: &mut VecDeque<String>,
        jinja: &JinjaEngine,
    ) -> TryParseFilterResult {
        if tokens.front().map(String::as_str) != Some(KIND) {
            return TryParseFilterResult::NotSupported;
        }

        tokens.pop_front();

        let key = match tokens.pop_front() {
            Some(k) => k,
            None => return TryParseFilterResult::Error(anyhow!("delete filter: missing key")),
        };

        let mut args = HashMap::new();
        args.insert("key".to_string(), key);

        TryParseFilterResult::Success(Stage::new(
            StageKind::Filter(Box::new(self.clone())),
            args,
            jinja.clone(),
        ))
    }
}

impl Filter for DeleteFilter {
    fn apply(&self, args: &HashMap<String, String>, context: &mut StageExecutionContext) -> Result<()> {
        let key = args
            .get("key")
            .ok_or_else(|| anyhow!("Delete filter: key is not specified"))?;

        let parts = hashmap_parse_key_parts(key);

        if let Value::Object(map) = &mut context.current_config {
            // We need to take ownership of the map to modify it if we use our utility
            let original_map = std::mem::take(map);
            let updated_map = hashmap_delete_nested_value(original_map, &parts);
            *map = updated_map;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_delete_filter_apply() {
        let filter = DeleteFilter;
        let mut args = HashMap::new();
        args.insert("key".to_string(), "a.b".to_string());

        let mut context = StageExecutionContext {
            current_config: json!({
                "a": {
                    "b": 1,
                    "c": 2
                },
                "x": 3
            }),
        };

        filter.apply(&args, &mut context).unwrap();

        assert_eq!(
            context.current_config,
            json!({
                "a": {
                    "c": 2
                },
                "x": 3
            })
        );
    }
}
