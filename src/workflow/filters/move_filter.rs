use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    utils::hashmap::{
        hashmap_extract_nested_value, hashmap_insert_nested_value, hashmap_parse_key_parts,
    },
    workflow::filters::Filter,
    workflow::stage::{StageArgs, StageExecutionContext},
};

pub const KIND: &str = "move";

/// Filter that moves a parameter from one key to another in the merged configuration.
/// Suffix '_filter' is added to file name because 'move' is a reserved word which causes issues with 'pub mod' statement
#[derive(Clone)]
pub struct MoveFilter {
    pub source: String,
    pub destination: String,
}

impl MoveFilter {
    pub fn new_from_args(tokens: StageArgs) -> Result<Box<dyn Filter>> {
        let source = match tokens.args.first() {
            Some(s) => s.clone(),
            None => return Err(anyhow!("move filter: missing source key")),
        };

        let destination = match tokens.args.get(1) {
            Some(d) => d.clone(),
            None => return Err(anyhow!("move filter: missing destination key")),
        };

        Ok(Box::new(MoveFilter {
            source,
            destination,
        }))
    }
}

impl Filter for MoveFilter {
    fn apply(&self, context: &mut StageExecutionContext) -> Result<()> {
        let source = &self.source;
        let destination = &self.destination;

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
        let filter = MoveFilter {
            source: "a".to_string(),
            destination: "b".to_string(),
        };

        let mut context = StageExecutionContext {
            current_config: json!({
                "a": 1,
                "x": 2
            }),
            ..Default::default()
        };

        filter.apply(&mut context).unwrap();

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
        let filter = MoveFilter {
            source: "a.b".to_string(),
            destination: "c.d".to_string(),
        };

        let mut context = StageExecutionContext {
            current_config: json!({
                "a": {
                    "b": 1
                }
            }),
            ..Default::default()
        };

        filter.apply(&mut context).unwrap();

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
        let filter = MoveFilter {
            source: ".".to_string(),
            destination: "config".to_string(),
        };

        let mut context = StageExecutionContext {
            current_config: json!({
                "a": 1,
                "b": 2
            }),
            ..Default::default()
        };

        filter.apply(&mut context).unwrap();

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
        let filter = MoveFilter {
            source: "config".to_string(),
            destination: ".".to_string(),
        };

        let mut context = StageExecutionContext {
            current_config: json!({
                "config": {
                    "a": 1,
                    "b": 2
                },
                "other": 3
            }),
            ..Default::default()
        };

        filter.apply(&mut context).unwrap();

        assert_eq!(
            context.current_config,
            json!({
                "a": 1,
                "b": 2
            })
        );
    }
}
