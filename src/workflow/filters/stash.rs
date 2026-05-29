use std::collections::{HashMap, VecDeque};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    jinja::JinjaEngine,
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

        let key = match tokens.pop_front() {
            Some(k) => k,
            None => return TryParseFilterResult::Error(anyhow!("stash filter: missing key")),
        };

        let mut args = HashMap::new();
        args.insert("mode".to_string(), mode);
        args.insert("key".to_string(), key);

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
                let value = std::mem::replace(
                    &mut context.current_config,
                    Value::Object(Default::default()),
                );
                context.stash.insert(key.clone(), value);
            }
            "pop" => {
                let value = context
                    .stash
                    .remove(key)
                    .ok_or_else(|| anyhow!("stash filter: key '{}' does not exist", key))?;
                context.current_config = value;
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
    fn test_pop_missing_key_errors() {
        let filter = StashFilter;
        let mut context = StageExecutionContext::new();
        let result = filter.apply(&make_args("pop", "missing"), &mut context);
        assert!(result.is_err());
    }
}
