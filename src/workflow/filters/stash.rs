use std::collections::{HashMap, VecDeque};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    jinja::JinjaEngine,
    utils::hashmap::{
        hashmap_delete_nested_value, hashmap_extract_nested_value, hashmap_insert_nested_value,
        hashmap_parse_key_parts,
    },
    workflow::filters::{BaseFilter, Filter, TryParseFilterResult},
    workflow::stage::{Stage, StageExecutionContext, StageKind},
};

const KIND: &str = "stash";

/// Filter that pushes/pops the current configuration to/from a named stash.
#[derive(Clone)]
pub struct StashFilter;

impl BaseFilter for StashFilter {
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

        let mode = match tokens.pop_front() {
            Some(m) => m,
            None => {
                return TryParseFilterResult::Error(anyhow!(
                    "stash filter: missing mode (push|pop)"
                ));
            }
        };

        if mode != "push" && mode != "pop" {
            return TryParseFilterResult::Error(anyhow!(
                "stash filter: unknown mode '{}', expected push or pop",
                mode
            ));
        }

        let mut args = HashMap::new();
        args.insert("mode".to_string(), mode.clone());

        if mode == "push" {
            if tokens.is_empty() {
                return TryParseFilterResult::Error(anyhow!("stash filter: missing destination"));
            }

            // The last one is the key
            let key = tokens.pop_back().unwrap();
            args.insert("key".to_string(), key);

            // The rest are flags
            while let Some(token) = tokens.pop_front() {
                if token.starts_with("--source=") {
                    args.insert(
                        "source".to_string(),
                        token.trim_start_matches("--source=").to_string(),
                    );
                } else if token == "--preserve" {
                    args.insert("preserve".to_string(), "true".to_string());
                } else {
                    return TryParseFilterResult::Error(anyhow!(
                        "stash filter: unknown argument '{}'",
                        token
                    ));
                }
            }
        } else {
            // mode == "pop"
            while let Some(token) = tokens.front() {
                if token == "--preserve" {
                    args.insert("preserve".to_string(), "true".to_string());
                    tokens.pop_front();
                } else {
                    break;
                }
            }

            let key = match tokens.pop_front() {
                Some(k) => k,
                None => return TryParseFilterResult::Error(anyhow!("stash filter: missing key")),
            };
            args.insert("key".to_string(), key);

            if let Some(dest) = tokens.pop_front() {
                args.insert("dest".to_string(), dest);
            }

            if !tokens.is_empty() {
                return TryParseFilterResult::Error(anyhow!(
                    "stash filter: unknown argument '{}'",
                    tokens.front().unwrap()
                ));
            }
        }

        TryParseFilterResult::Success(Stage::new(
            StageKind::Filter(Box::new(self.clone())),
            args,
            jinja.clone(),
        ))
    }
}

impl Filter for StashFilter {
    fn apply(
        &self,
        args: &HashMap<String, String>,
        context: &mut StageExecutionContext,
    ) -> Result<()> {
        let mode = args
            .get("mode")
            .ok_or_else(|| anyhow!("stash filter: mode is not specified"))?;
        let key = args
            .get("key")
            .ok_or_else(|| anyhow!("stash filter: key is not specified"))?;

        match mode.as_str() {
            "push" => {
                if context.stash.contains_key(key) {
                    return Err(anyhow!("stash filter: key '{}' already exists", key));
                }

                let source = args.get("source");
                let preserve = args.get("preserve").map(|v| v == "true").unwrap_or(false);

                if let Some(s) = source {
                    let parts = hashmap_parse_key_parts(s);
                    let current_obj = context
                        .current_config
                        .as_object()
                        .ok_or_else(|| anyhow!("stash filter: current_config is not an object"))?
                        .clone();

                    let (new_map, extracted) = hashmap_extract_nested_value(current_obj, &parts);
                    let value = extracted
                        .ok_or_else(|| anyhow!("stash filter: source '{}' not found", s))?;

                    context.stash.insert(key.clone(), value);

                    if !preserve {
                        context.current_config = Value::Object(new_map);
                    }
                } else {
                    // If no source is provided, try to use key as a source if it exists in the config
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
                }
            }
            "pop" => {
                let preserve = args.get("preserve").map(|v| v == "true").unwrap_or(false);
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

                match args.get("dest") {
                    None => context.current_config = value,
                    Some(dest) => {
                        let parts = hashmap_parse_key_parts(dest);
                        let mut map =
                            match std::mem::replace(&mut context.current_config, Value::Null) {
                                Value::Object(m) => m,
                                _ => serde_json::Map::new(),
                            };

                        let (_, existing) = hashmap_extract_nested_value(map.clone(), &parts);
                        let mut target_val =
                            existing.unwrap_or(Value::Object(serde_json::Map::new()));

                        crate::utils::cfg_values::cfg_values_deep_merge(&mut target_val, &value)?;
                        map = hashmap_insert_nested_value(map, &parts, target_val);
                        context.current_config = Value::Object(map);
                    }
                }
            }
            _ => return Err(anyhow!("stash filter: unknown mode '{}'", mode)),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_args(mode: &str, key: &str) -> HashMap<String, String> {
        let mut args = HashMap::new();
        args.insert("mode".to_string(), mode.to_string());
        args.insert("key".to_string(), key.to_string());
        args
    }

    #[test]
    fn test_push_stores_config_and_clears() {
        let filter = StashFilter;
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});

        filter
            .apply(&make_args("push", "saved"), &mut context)
            .unwrap();

        assert_eq!(context.stash["saved"], json!({"a": 1}));
        assert_eq!(context.current_config, json!({}));
    }

    #[test]
    fn test_push_with_preserve_keeps_config() {
        let filter = StashFilter;
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});

        let mut args = make_args("push", "saved");
        args.insert("preserve".to_string(), "true".to_string());
        filter.apply(&args, &mut context).unwrap();

        assert_eq!(context.stash["saved"], json!({"a": 1}));
        assert_eq!(context.current_config, json!({"a": 1}));
    }

    #[test]
    fn test_push_with_source_extracts_subproperty() {
        let filter = StashFilter;
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": {"b": 1}, "x": 2});

        let mut args = make_args("push", "saved");
        args.insert("source".to_string(), "a.b".to_string());
        filter.apply(&args, &mut context).unwrap();

        assert_eq!(context.stash["saved"], json!(1));
        // Note: a becomes empty and is removed by hashmap_extract_nested_value
        assert_eq!(context.current_config, json!({"x": 2}));
    }

    #[test]
    fn test_push_with_source_and_preserve() {
        let filter = StashFilter;
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": {"b": 1}, "x": 2});

        let mut args = make_args("push", "saved");
        args.insert("source".to_string(), "a.b".to_string());
        args.insert("preserve".to_string(), "true".to_string());
        filter.apply(&args, &mut context).unwrap();

        assert_eq!(context.stash["saved"], json!(1));
        assert_eq!(context.current_config, json!({"a": {"b": 1}, "x": 2}));
    }

    #[test]
    fn test_push_duplicate_key_errors() {
        let filter = StashFilter;
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});
        filter
            .apply(&make_args("push", "saved"), &mut context)
            .unwrap();

        context.current_config = json!({"b": 2});
        let result = filter.apply(&make_args("push", "saved"), &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_parse_args_push_with_flags() {
        let filter = StashFilter;
        let jinja = JinjaEngine::new();
        let mut tokens = VecDeque::from(vec![
            "stash".to_string(),
            "push".to_string(),
            "--source=a.b".to_string(),
            "--preserve".to_string(),
            "mykey".to_string(),
        ]);

        let result = filter.try_parse_args(&mut tokens, &jinja);
        match result {
            TryParseFilterResult::Success(stage) => {
                assert_eq!(stage.args.get("mode").unwrap(), "push");
                assert_eq!(stage.args.get("key").unwrap(), "mykey");
                assert_eq!(stage.args.get("source").unwrap(), "a.b");
                assert_eq!(stage.args.get("preserve").unwrap(), "true");
            }
            _ => panic!("Expected Success"),
        }
    }

    #[test]
    fn test_try_parse_args_push_destination_not_last_fails() {
        let filter = StashFilter;
        let jinja = JinjaEngine::new();
        let mut tokens = VecDeque::from(vec![
            "stash".to_string(),
            "push".to_string(),
            "mykey".to_string(),
            "--preserve".to_string(),
        ]);

        let result = filter.try_parse_args(&mut tokens, &jinja);
        match result {
            TryParseFilterResult::Error(e) => {
                assert!(e.to_string().contains("unknown argument 'mykey'"));
            }
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_push_auto_unwraps_key() {
        let filter = StashFilter;
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"saved": {"a": 1}, "x": 2});

        filter
            .apply(&make_args("push", "saved"), &mut context)
            .unwrap();

        assert_eq!(context.stash["saved"], json!({"a": 1}));
        assert_eq!(context.current_config, json!({"x": 2}));
    }

    #[test]
    fn test_pop_flattens_values() {
        let filter = StashFilter;
        let mut context = StageExecutionContext::new();
        context
            .stash
            .insert("saved".to_string(), json!({"values": {"a": 1}, "b": 2}));

        filter
            .apply(&make_args("pop", "saved"), &mut context)
            .unwrap();

        assert_eq!(context.current_config, json!({"a": 1, "b": 2}));
        assert!(!context.stash.contains_key("saved"));
    }

    #[test]
    fn test_pop_restores_config() {
        let filter = StashFilter;
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});
        filter
            .apply(&make_args("push", "saved"), &mut context)
            .unwrap();

        filter
            .apply(&make_args("pop", "saved"), &mut context)
            .unwrap();

        assert_eq!(context.current_config, json!({"a": 1}));
        assert!(!context.stash.contains_key("saved"));
    }

    #[test]
    fn test_pop_with_preserve_keeps_in_stash() {
        let filter = StashFilter;
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});
        filter
            .apply(&make_args("push", "saved"), &mut context)
            .unwrap();

        let mut args = make_args("pop", "saved");
        args.insert("preserve".to_string(), "true".to_string());
        filter.apply(&args, &mut context).unwrap();

        assert_eq!(context.current_config, json!({"a": 1}));
        assert_eq!(context.stash["saved"], json!({"a": 1}));
    }

    #[test]
    fn test_try_parse_args_pop_with_preserve() {
        let filter = StashFilter;
        let jinja = JinjaEngine::new();
        let mut tokens = VecDeque::from(vec![
            "stash".to_string(),
            "pop".to_string(),
            "--preserve".to_string(),
            "mykey".to_string(),
            "dest.path".to_string(),
        ]);

        let result = filter.try_parse_args(&mut tokens, &jinja);
        match result {
            TryParseFilterResult::Success(stage) => {
                assert_eq!(stage.args.get("mode").unwrap(), "pop");
                assert_eq!(stage.args.get("key").unwrap(), "mykey");
                assert_eq!(stage.args.get("dest").unwrap(), "dest.path");
                assert_eq!(stage.args.get("preserve").unwrap(), "true");
            }
            _ => panic!("Expected Success"),
        }
    }

    #[test]
    fn test_pop_missing_key_errors() {
        let filter = StashFilter;
        let mut context = StageExecutionContext::new();
        let result = filter.apply(&make_args("pop", "missing"), &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn test_pop_with_dest_inserts_into_current_config() {
        let filter = StashFilter;
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});
        filter
            .apply(&make_args("push", "saved"), &mut context)
            .unwrap();

        context.current_config = json!({"x": 10});
        let mut args = make_args("pop", "saved");
        args.insert("dest".to_string(), "restored".to_string());
        filter.apply(&args, &mut context).unwrap();

        assert_eq!(
            context.current_config,
            json!({"x": 10, "restored": {"a": 1}})
        );
        assert!(!context.stash.contains_key("saved"));
    }

    #[test]
    fn test_pop_with_dest_errors_if_dest_exists() {
        let filter = StashFilter;
        let mut context = StageExecutionContext::new();
        context.current_config = json!({"a": 1});
        filter
            .apply(&make_args("push", "saved"), &mut context)
            .unwrap();

        context.current_config = json!({"restored": 99});
        let mut args = make_args("pop", "saved");
        args.insert("dest".to_string(), "restored".to_string());
        let result = filter.apply(&args, &mut context);
        assert!(result.is_err());
    }
}
