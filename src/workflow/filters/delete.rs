use std::collections::VecDeque;

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    utils::hashmap::{hashmap_delete_nested_value, hashmap_parse_key_parts},
    workflow::filters::Filter,
    workflow::stage::StageExecutionContext,
};

pub const KIND: &str = "delete";

/// Filter that deletes a parameter from the merged configuration.
#[derive(Clone)]
pub struct DeleteFilter {
    pub key: String,
}

impl DeleteFilter {
    pub fn new_from_args(mut args: VecDeque<String>) -> Result<Box<dyn Filter>> {
        let key = match args.pop_front() {
            Some(k) => k,
            None => return Err(anyhow!("delete filter: missing key")),
        };

        Ok(Box::new(DeleteFilter { key: String::from(key) }))
    }
}

impl Filter for DeleteFilter {
    fn apply(
        &self,
        context: &mut StageExecutionContext,
    ) -> Result<()> {
        let parts = hashmap_parse_key_parts(self.key.as_str());

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
    use std::collections::HashMap;
    use serde_json::json;

    #[test]
    fn test_delete_filter_apply() {
        let filter = DeleteFilter { key: String::new() };
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
            ..Default::default()
        };

        filter.apply(&mut context).unwrap();

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
