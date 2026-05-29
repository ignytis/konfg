use std::collections::{HashMap, VecDeque};

use anyhow::{Result, anyhow};
use serde_json::{Map, Value};

use crate::{
    jinja::JinjaEngine,
    utils::hashmap::{hashmap_insert_nested_value, hashmap_parse_key_parts},
    workflow::io::{BaseIoHandler, InputHandler, TryParseResult},
    workflow::stage::{Stage, StageExecutionContext, StageKind},
};

const KIND: &str = "param";

/// Handles single key-value pair input operations.
#[derive(Clone)]
pub struct ParamHandler;

impl BaseIoHandler for ParamHandler {
    fn supports(&self, kind: &str) -> bool {
        kind == KIND
    }

    fn clone_box(&self) -> Box<dyn BaseIoHandler> {
        Box::new(self.clone())
    }

    fn try_parse_args(
        &self,
        tokens: &mut VecDeque<String>,
        jinja: &JinjaEngine,
        is_output: bool,
    ) -> TryParseResult {
        if tokens.front().map(String::as_str) != Some(KIND) {
            return TryParseResult::NotSupported;
        }

        if is_output {
            return TryParseResult::Error(anyhow!("param: writing to params is not supported"));
        }

        tokens.pop_front();

        let key = match tokens.pop_front() {
            Some(k) => k,
            None => return TryParseResult::Error(anyhow!("param: missing key")),
        };

        let value = match tokens.pop_front() {
            Some(v) => v,
            None => return TryParseResult::Error(anyhow!("param: missing value")),
        };

        let mut args = HashMap::new();
        args.insert("key".to_string(), key);
        args.insert("value".to_string(), value);

        TryParseResult::Success(Stage::new(
            StageKind::Input(Box::new(self.clone())),
            args,
            jinja.clone(),
        ))
    }
}

impl InputHandler for ParamHandler {
    fn read(
        &self,
        args: &HashMap<String, String>,
        _jinja: &JinjaEngine,
        _context: &StageExecutionContext,
    ) -> Result<Value> {
        let key = args
            .get("key")
            .ok_or_else(|| anyhow!("Param handler: key is not specified"))?;
        let value = args
            .get("value")
            .ok_or_else(|| anyhow!("Param handler: value is not specified"))?;

        Ok(self.parse_nested_param(key, value))
    }
}

impl ParamHandler {
    /// Parses a key-value pair into a nested `Value` structure.
    /// Dots in the key are treated as level separators, unless they are escaped (doubled).
    fn parse_nested_param(&self, key: &str, value: &str) -> Value {
        let parts = hashmap_parse_key_parts(key);
        let map = hashmap_insert_nested_value(Map::new(), &parts, Value::String(value.to_string()));
        Value::Object(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_param_read_simple() {
        let handler = ParamHandler;
        let mut args = HashMap::new();
        args.insert("key".to_string(), "my_key".to_string());
        args.insert("value".to_string(), "my_value".to_string());

        let jinja = JinjaEngine::new();
        let context = StageExecutionContext {
            current_config: Value::Object(Default::default()),
        };

        let content = handler.read(&args, &jinja, &context).unwrap();
        assert_eq!(content, json!({"my_key": "my_value"}));
    }

    #[test]
    fn test_param_read_nested() {
        let handler = ParamHandler;
        let mut args = HashMap::new();
        args.insert("key".to_string(), "a.b.c".to_string());
        args.insert("value".to_string(), "val".to_string());

        let jinja = JinjaEngine::new();
        let context = StageExecutionContext {
            current_config: Value::Object(Default::default()),
        };

        let content = handler.read(&args, &jinja, &context).unwrap();
        assert_eq!(
            content,
            json!({
                "a": {
                    "b": {
                        "c": "val"
                    }
                }
            })
        );
    }

    #[test]
    fn test_param_read_escaped() {
        let handler = ParamHandler;
        let mut args = HashMap::new();
        args.insert("key".to_string(), "a..b.c".to_string());
        args.insert("value".to_string(), "val".to_string());

        let jinja = JinjaEngine::new();
        let context = StageExecutionContext {
            current_config: Value::Object(Default::default()),
        };

        let content = handler.read(&args, &jinja, &context).unwrap();
        assert_eq!(
            content,
            json!({
                "a.b": {
                    "c": "val"
                }
            })
        );
    }

    #[test]
    fn test_param_read_multiple_escaped() {
        let handler = ParamHandler;
        let mut args = HashMap::new();
        args.insert("key".to_string(), "a....b".to_string());
        args.insert("value".to_string(), "val".to_string());

        let jinja = JinjaEngine::new();
        let context = StageExecutionContext {
            current_config: Value::Object(Default::default()),
        };

        let content = handler.read(&args, &jinja, &context).unwrap();
        assert_eq!(content, json!({"a..b": "val"}));
    }

    #[test]
    fn test_param_read_escaped_at_end() {
        let handler = ParamHandler;
        let mut args = HashMap::new();
        args.insert("key".to_string(), "a..".to_string());
        args.insert("value".to_string(), "val".to_string());

        let jinja = JinjaEngine::new();
        let context = StageExecutionContext {
            current_config: Value::Object(Default::default()),
        };

        let content = handler.read(&args, &jinja, &context).unwrap();
        assert_eq!(content, json!({"a.": "val"}));
    }

    #[test]
    fn test_param_read_triple_dot() {
        let handler = ParamHandler;
        let mut args = HashMap::new();
        args.insert("key".to_string(), "a...b".to_string());
        args.insert("value".to_string(), "val".to_string());

        let jinja = JinjaEngine::new();
        let context = StageExecutionContext {
            current_config: Value::Object(Default::default()),
        };

        let content = handler.read(&args, &jinja, &context).unwrap();
        assert_eq!(
            content,
            json!({
                "a.": {
                    "b": "val"
                }
            })
        );
    }

    #[test]
    fn test_param_try_parse_args_output_error() {
        let handler = ParamHandler;
        let mut tokens = VecDeque::from(vec![
            "param".to_string(),
            "key".to_string(),
            "value".to_string(),
        ]);
        let jinja = JinjaEngine::new();
        let result = handler.try_parse_args(&mut tokens, &jinja, true);
        assert!(matches!(result, TryParseResult::Error(_)));
        if let TryParseResult::Error(e) = result {
            assert_eq!(e.to_string(), "param: writing to params is not supported");
        }
    }

    #[test]
    fn test_param_supports() {
        let handler = ParamHandler;
        assert!(handler.supports("param"));
        assert!(!handler.supports("env"));
    }

    #[test]
    fn test_param_try_parse_args() {
        let handler = ParamHandler;
        let mut tokens = VecDeque::from(vec![
            "param".to_string(),
            "foo".to_string(),
            "bar".to_string(),
        ]);
        let jinja = JinjaEngine::new();

        let result = handler.try_parse_args(&mut tokens, &jinja, false);
        if let TryParseResult::Success(stage) = result {
            assert_eq!(stage.args.get("key").unwrap(), "foo");
            assert_eq!(stage.args.get("value").unwrap(), "bar");
        } else {
            panic!("Expected Success");
        }
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_param_try_parse_args_missing_value() {
        let handler = ParamHandler;
        let mut tokens = VecDeque::from(vec!["param".to_string(), "foo".to_string()]);
        let jinja = JinjaEngine::new();

        let result = handler.try_parse_args(&mut tokens, &jinja, false);
        if let TryParseResult::Error(e) = result {
            assert_eq!(e.to_string(), "param: missing value");
        } else {
            panic!("Expected Error");
        }
    }
}
