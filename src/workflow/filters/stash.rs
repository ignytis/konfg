use std::collections::HashMap;

use anyhow::{Ok, Result, anyhow};
use serde_json::Value;

use crate::{
    utils::{
        cfg_values::cfg_values_deep_merge,
        hashmap::{
            hashmap_delete_nested_value, hashmap_extract_nested_value, hashmap_insert_nested_value,
            hashmap_parse_key_parts,
        },
    },
    workflow::{
        filters::Filter,
        stage::{StageArgs, StageExecutionContext},
    },
};

pub const KIND: &str = "stash";

#[derive(Clone, Debug, PartialEq)]
pub enum StashMode {
    Push,
    Pop,
}

impl TryFrom<&str> for StashMode {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "push" => Ok(StashMode::Push),
            "pop" => Ok(StashMode::Pop),
            _ => Err(anyhow!("stash filter: unsupported mode '{}'", value)),
        }
    }
}

/// Filter that pushes/pops a parameter to/from the stash.
#[derive(Clone)]
pub struct StashFilter {
    pub mode: StashMode,
    pub key: String,
    pub source: Option<String>,
    pub destination: Option<String>,
    pub preserve: bool,
}

impl StashFilter {
    pub fn new_from_args(tokens: StageArgs) -> Result<Box<dyn Filter>> {
        let mode_str = match tokens.args.first() {
            Some(m) => m,
            None => return Err(anyhow!("stash filter: missing mode (push|pop)")),
        };

        let mode: StashMode = mode_str.as_str().try_into()?;
        let key;
        let mut source = tokens.kwargs.get("--source").cloned();
        if source.is_none() {
            source = tokens.kwargs.get("source").cloned();
        }
        let mut destination = None;
        let mut preserve = false;

        // index 0 is the mode; positional args start at 1
        let mut idx = 1;

        if mode == StashMode::Push {
            if idx >= tokens.args.len() {
                return Err(anyhow!("stash filter: missing destination"));
            }

            // The last one is the key
            key = tokens.args.last().unwrap().clone();

            // The rest (between idx and last) are flags
            while idx < tokens.args.len() - 1 {
                let token = &tokens.args[idx];
                if token == "--preserve" {
                    preserve = true;
                } else {
                    return Err(anyhow!("stash filter: unknown argument '{}'", token));
                }
                idx += 1;
            }
        } else {
            // mode == Pop: first consume --preserve flags, then key, then optional destination
            while idx < tokens.args.len() && tokens.args[idx] == "--preserve" {
                preserve = true;
                idx += 1;
            }

            key = match tokens.args.get(idx) {
                Some(k) => k.clone(),
                None => return Err(anyhow!("stash filter: missing key")),
            };
            idx += 1;

            if let Some(dest) = tokens.args.get(idx) {
                destination = Some(dest.clone());
                idx += 1;
            }

            if idx < tokens.args.len() {
                return Err(anyhow!(
                    "stash filter: unknown argument '{}'",
                    tokens.args[idx]
                ));
            }
        }

        // Ensure there are no unknown kwargs
        for k in tokens.kwargs.keys() {
            if k != "--source" && k != "source" {
                return Err(anyhow!("stash filter: unknown argument '{}'", k));
            }
        }

        Ok(Box::new(StashFilter {
            mode,
            key,
            source,
            destination,
            preserve,
        }))
    }

    fn _push(&self, context: &mut StageExecutionContext) -> Result<()> {
        let key = &self.key;
        if context.stash.contains_key(key) {
            return Err(anyhow!("stash filter: key '{}' already exists", key));
        }

        let preserve = self.preserve;

        // If no source is provided, try to use key as a source if it exists in the config
        if self.source.is_none() {
            let parts = hashmap_parse_key_parts(key);
            let extracted = if let Value::Object(map) = &context.current_config {
                let (_, val) = hashmap_extract_nested_value(map.clone(), &parts);
                val
            } else {
                None
            };

            if let Some(value) = extracted {
                context.stash.insert(key.clone(), value);
                if !preserve {
                    if let Value::Object(map) = &mut context.current_config {
                        let original_map = std::mem::take(map);
                        *map = hashmap_delete_nested_value(original_map, &parts);
                    }
                }
            } else {
                let value = if preserve {
                    context.current_config.clone()
                } else {
                    std::mem::replace(
                        &mut context.current_config,
                        Value::Object(Default::default()),
                    )
                };
                context.stash.insert(key.clone(), value);
            }
            return Ok(());
        }

        let s = self.source.as_ref().unwrap().as_str();
        let parts = hashmap_parse_key_parts(s);
        let current_obj = context
            .current_config
            .as_object()
            .ok_or_else(|| anyhow!("stash filter: current_config is not an object"))?
            .clone();

        let (new_map, extracted) = hashmap_extract_nested_value(current_obj, &parts);
        let value = extracted.ok_or_else(|| anyhow!("stash filter: source '{}' not found", s))?;

        context.stash.insert(key.clone(), value);

        if !preserve {
            context.current_config = Value::Object(new_map);
        }
        Ok(())
    }

    fn _pop(&self, context: &mut StageExecutionContext) -> Result<()> {
        let key = &self.key;
        let preserve = self.preserve;
        let mut value = if preserve {
            context.stash.get(key).cloned()
        } else {
            context.stash.remove(key)
        }
        .ok_or_else(|| anyhow!("stash filter: key '{}' does not exist", key))?;

        // Flatten "values" property if it exists in the popped object to eliminate extra nesting
        if let Value::Object(mut map) = value {
            if let Some(inner) = map.remove("values") {
                if let Value::Object(inner_map) = inner {
                    for (k, v) in inner_map {
                        // If there's a collision, the original value (not from "values") wins
                        // but usually there's no collision in this workflow
                        map.entry(k).or_insert(v);
                    }
                } else {
                    // If it's not an object, put it back
                    map.insert("values".to_string(), inner);
                }
            }
            value = Value::Object(map);
        }

        if self.destination.is_none() {
            context.current_config = value;
            return Ok(());
        }

        let dest = self.destination.as_ref().unwrap().as_str();
        let parts = hashmap_parse_key_parts(dest);
        let mut map = match std::mem::replace(&mut context.current_config, Value::Null) {
            Value::Object(m) => m,
            _ => serde_json::Map::new(),
        };

        let (_, existing) = hashmap_extract_nested_value(map.clone(), &parts);
        let mut target_val = existing.unwrap_or(Value::Object(serde_json::Map::new()));

        // TODO: why deep merge here? Do we really want to merge the extracted value?
        cfg_values_deep_merge(&mut target_val, &value, &HashMap::new())?;
        map = hashmap_insert_nested_value(map, &parts, target_val);
        context.current_config = Value::Object(map);
        Ok(())
    }
}

impl Filter for StashFilter {
    fn apply(&self, context: &mut StageExecutionContext) -> Result<()> {
        match self.mode {
            StashMode::Push => self._push(context),
            StashMode::Pop => self._pop(context),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_push_stores_config_and_clears() {
        let filter = StashFilter {
            mode: StashMode::Push,
            key: "saved".to_string(),
            source: None,
            destination: None,
            preserve: false,
        };
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});

        filter.apply(&mut context).unwrap();

        assert_eq!(context.stash["saved"], json!({"a": 1}));
        assert_eq!(context.current_config, json!({}));
    }

    #[test]
    fn test_push_with_preserve_keeps_config() {
        let filter = StashFilter {
            mode: StashMode::Push,
            key: "saved".to_string(),
            source: None,
            destination: None,
            preserve: true,
        };
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});

        filter.apply(&mut context).unwrap();

        assert_eq!(context.stash["saved"], json!({"a": 1}));
        assert_eq!(context.current_config, json!({"a": 1}));
    }

    #[test]
    fn test_push_with_source_extracts_subproperty() {
        let filter = StashFilter {
            mode: StashMode::Push,
            key: "saved".to_string(),
            source: Some("a.b".to_string()),
            destination: None,
            preserve: false,
        };
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": {"b": 1}, "x": 2});

        filter.apply(&mut context).unwrap();

        assert_eq!(context.stash["saved"], json!(1));
        // Note: a becomes empty and is removed by hashmap_extract_nested_value
        assert_eq!(context.current_config, json!({"x": 2}));
    }

    #[test]
    fn test_push_with_source_and_preserve() {
        let filter = StashFilter {
            mode: StashMode::Push,
            key: "saved".to_string(),
            source: Some("a.b".to_string()),
            destination: None,
            preserve: true,
        };
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": {"b": 1}, "x": 2});

        filter.apply(&mut context).unwrap();

        assert_eq!(context.stash["saved"], json!(1));
        assert_eq!(context.current_config, json!({"a": {"b": 1}, "x": 2}));
    }

    #[test]
    fn test_push_duplicate_key_errors() {
        let filter = StashFilter {
            mode: StashMode::Push,
            key: "saved".to_string(),
            source: None,
            destination: None,
            preserve: false,
        };
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});
        filter.apply(&mut context).unwrap();

        context.current_config = json!({"b": 2});
        let result = filter.apply(&mut context);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_from_args_push_with_flags() {
        let args = StageArgs::new_from_args(vec![
            "push".to_string(),
            "--source=a.b".to_string(),
            "--preserve".to_string(),
            "mykey".to_string(),
        ]);

        let _filter_box = StashFilter::new_from_args(args).unwrap();
    }

    #[test]
    fn test_new_from_args_push_destination_not_last_fails() {
        let args = StageArgs::new_from_args(vec![
            "push".to_string(),
            "mykey".to_string(),
            "--preserve".to_string(),
        ]);

        let result = StashFilter::new_from_args(args);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("unknown argument 'mykey'"));
        }
    }

    #[test]
    fn test_push_auto_unwraps_key() {
        let filter = StashFilter {
            mode: StashMode::Push,
            key: "saved".to_string(),
            source: None,
            destination: None,
            preserve: false,
        };
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"saved": {"a": 1}, "x": 2});

        filter.apply(&mut context).unwrap();

        assert_eq!(context.stash["saved"], json!({"a": 1}));
        assert_eq!(context.current_config, json!({"x": 2}));
    }

    #[test]
    fn test_pop_flattens_values() {
        let filter = StashFilter {
            mode: StashMode::Pop,
            key: "saved".to_string(),
            source: None,
            destination: None,
            preserve: false,
        };
        let mut context = StageExecutionContext::new();
        context
            .stash
            .insert("saved".to_string(), json!({"values": {"a": 1}, "b": 2}));

        filter.apply(&mut context).unwrap();

        assert_eq!(context.current_config, json!({"a": 1, "b": 2}));
        assert!(!context.stash.contains_key("saved"));
    }

    #[test]
    fn test_pop_restores_config() {
        let push_filter = StashFilter {
            mode: StashMode::Push,
            key: "saved".to_string(),
            source: None,
            destination: None,
            preserve: false,
        };
        let pop_filter = StashFilter {
            mode: StashMode::Pop,
            key: "saved".to_string(),
            source: None,
            destination: None,
            preserve: false,
        };
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});
        push_filter.apply(&mut context).unwrap();
        pop_filter.apply(&mut context).unwrap();

        assert_eq!(context.current_config, json!({"a": 1}));
        assert!(!context.stash.contains_key("saved"));
    }

    #[test]
    fn test_pop_with_preserve_keeps_in_stash() {
        let push_filter = StashFilter {
            mode: StashMode::Push,
            key: "saved".to_string(),
            source: None,
            destination: None,
            preserve: false,
        };
        let pop_filter = StashFilter {
            mode: StashMode::Pop,
            key: "saved".to_string(),
            source: None,
            destination: None,
            preserve: true,
        };
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});
        push_filter.apply(&mut context).unwrap();
        pop_filter.apply(&mut context).unwrap();

        assert_eq!(context.current_config, json!({"a": 1}));
        assert_eq!(context.stash["saved"], json!({"a": 1}));
    }

    #[test]
    fn test_new_from_args_pop_with_preserve() {
        let args = StageArgs::new_from_args(vec![
            "pop".to_string(),
            "--preserve".to_string(),
            "mykey".to_string(),
            "dest.path".to_string(),
        ]);

        let _ = StashFilter::new_from_args(args).unwrap();
    }

    #[test]
    fn test_pop_missing_key_errors() {
        let filter = StashFilter {
            mode: StashMode::Pop,
            key: "missing".to_string(),
            source: None,
            destination: None,
            preserve: false,
        };
        let mut context = StageExecutionContext::new();
        let result = filter.apply(&mut context);
        assert!(result.is_err());
    }

    #[test]
    fn test_pop_with_dest_inserts_into_current_config() {
        let push_filter = StashFilter {
            mode: StashMode::Push,
            key: "saved".to_string(),
            source: None,
            destination: None,
            preserve: false,
        };
        let pop_filter = StashFilter {
            mode: StashMode::Pop,
            key: "saved".to_string(),
            source: None,
            destination: Some("restored".to_string()),
            preserve: false,
        };
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});
        push_filter.apply(&mut context).unwrap();

        context.current_config = json!({"x": 10});
        pop_filter.apply(&mut context).unwrap();

        assert_eq!(
            context.current_config,
            json!({"x": 10, "restored": {"a": 1}})
        );
        assert!(!context.stash.contains_key("saved"));
    }

    #[test]
    fn test_pop_with_dest_errors_if_dest_exists() {
        let push_filter = StashFilter {
            mode: StashMode::Push,
            key: "saved".to_string(),
            source: None,
            destination: None,
            preserve: false,
        };
        let pop_filter = StashFilter {
            mode: StashMode::Pop,
            key: "saved".to_string(),
            source: None,
            destination: Some("restored".to_string()),
            preserve: false,
        };
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});
        push_filter.apply(&mut context).unwrap();

        context.current_config = json!({"restored": 99});
        let result = pop_filter.apply(&mut context);
        assert!(result.is_err());
    }
}
