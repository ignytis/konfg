use std::collections::{HashMap, VecDeque};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    jinja::JinjaEngine,
    utils::hashmap::{
        hashmap_extract_nested_value, hashmap_insert_nested_value, hashmap_parse_key_parts,
    },
    workflow::filters::{BaseFilter, Filter, TryParseFilterResult},
    workflow::stage::{Stage, StageExecutionContext, StageKind},
};

const KIND: &str = "move";

/// Filter that moves a parameter from one key to another in the merged configuration.
/// Suffix '_filter' is added to file name because 'move' is a reserved word which causes issues with 'pub mod' statement
#[derive(Clone)]
pub struct MoveFilter;

impl BaseFilter for MoveFilter {
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

        let source = match tokens.pop_front() {
            Some(s) => s,
            None => return TryParseFilterResult::Error(anyhow!("move filter: missing source key")),
        };

        let destination = match tokens.pop_front() {
            Some(d) => d,
            None => {
                return TryParseFilterResult::Error(anyhow!(
                    "move filter: missing destination key"
                ));
            }
        };

        let mut args = HashMap::new();
        args.insert("source".to_string(), source);
        args.insert("destination".to_string(), destination);

        TryParseFilterResult::Success(Stage::new(
            StageKind::Filter(Box::new(self.clone())),
            args,
            jinja.clone(),
        ))
    }
}

impl Filter for MoveFilter {
    fn apply(&self, args: &HashMap<String, String>, context: &mut StageExecutionContext) -> Result<()> {
        let source = args
            .get("source")
            .ok_or_else(|| anyhow!("Move filter: source is not specified"))?;
        let destination = args
            .get("destination")
            .ok_or_else(|| anyhow!("Move filter: destination is not specified"))?;

        if source == destination {
            return Ok(());
        }

        let value_to_move = if source == "." {
            Some(std::mem::replace(
                &mut context.current_config,
                Value::Object(serde_json::Map::new()),
            ))
        } else {
            let parts = hashmap_parse_key_parts(source);
            if let Value::Object(map) = &mut context.current_config {
                let original_map = std::mem::take(map);
                let (updated_map, val) = hashmap_extract_nested_value(original_map, &parts);
                *map = updated_map;
                val
            } else {
                None
            }
        };

        if let Some(val) = value_to_move {
            if destination == "." {
                context.current_config = val;
            } else {
                let dest_parts = hashmap_parse_key_parts(destination);
                if let Value::Object(map) = &mut context.current_config {
                    let original_map = std::mem::take(map);
                    let updated_map = hashmap_insert_nested_value(original_map, &dest_parts, val);
                    *map = updated_map;
                } else {
                    // If merged is not an object, we can't insert into it unless destination is "."
                    // which is handled above. If it's a scalar, we might want to wrap it or error.
                    // Given the context of konfg, merged is usually an object.
                    if context.current_config.is_null() {
                        let map = serde_json::Map::new();
                        let updated_map = hashmap_insert_nested_value(map, &dest_parts, val);
                        context.current_config = Value::Object(updated_map);
                    } else {
                        return Err(anyhow!(
                            "Move filter: cannot move to nested key in non-object configuration"
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_move_filter_apply_simple() {
        let filter = MoveFilter;
        let mut args = HashMap::new();
        args.insert("source".to_string(), "a".to_string());
        args.insert("destination".to_string(), "b".to_string());

        let mut context = StageExecutionContext {
            current_config: json!({
                "a": 1,
                "x": 2
            }),
        };

        filter.apply(&args, &mut context).unwrap();

        assert_eq!(
            context.current_config,
            json!({
                "b": 1,
                "x": 2
            })
        );
    }

    #[test]
    fn test_move_filter_apply_nested() {
        let filter = MoveFilter;
        let mut args = HashMap::new();
        args.insert("source".to_string(), "a.b".to_string());
        args.insert("destination".to_string(), "c.d".to_string());

        let mut context = StageExecutionContext {
            current_config: json!({
                "a": {
                    "b": 1
                }
            }),
        };

        filter.apply(&args, &mut context).unwrap();

        assert_eq!(
            context.current_config,
            json!({
                "c": {
                    "d": 1
                }
            })
        );
    }

    #[test]
    fn test_move_filter_apply_root_source() {
        let filter = MoveFilter;
        let mut args = HashMap::new();
        args.insert("source".to_string(), ".".to_string());
        args.insert("destination".to_string(), "config".to_string());

        let mut context = StageExecutionContext {
            current_config: json!({
                "a": 1,
                "b": 2
            }),
        };

        filter.apply(&args, &mut context).unwrap();

        assert_eq!(
            context.current_config,
            json!({
                "config": {
                    "a": 1,
                    "b": 2
                }
            })
        );
    }

    #[test]
    fn test_move_filter_apply_root_destination() {
        let filter = MoveFilter;
        let mut args = HashMap::new();
        args.insert("source".to_string(), "config".to_string());
        args.insert("destination".to_string(), ".".to_string());

        let mut context = StageExecutionContext {
            current_config: json!({
                "config": {
                    "a": 1,
                    "b": 2
                },
                "other": 3
            }),
        };

        filter.apply(&args, &mut context).unwrap();

        assert_eq!(
            context.current_config,
            json!({
                "a": 1,
                "b": 2
            })
        );
    }
}
