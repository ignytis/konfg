use std::collections::VecDeque;

use anyhow::{Result, anyhow};
use serde_json::{Map, Value};

use crate::{
    utils::hashmap::{hashmap_insert_nested_value, hashmap_parse_key_parts},
    workflow::io::{BaseIoHandler, InputHandler},
    workflow::stage::{Stage, StageArgs, StageExecutionContext, StageKind},
};

pub const KIND: &str = "param";

/// Handles single key-value pair input operations.
#[derive(Clone)]
pub struct ParamHandler {
    pub key: String,
    pub value: String,
}

impl ParamHandler {
    pub fn new_from_args(tokens: StageArgs, is_output: bool) -> Result<Stage> {
        let mut args = VecDeque::from(tokens.args);
        if args.front().map(String::as_str) != Some(KIND) {
            return Err(anyhow!("param handler: not supported"));
        }

        if is_output {
            return Err(anyhow!("param: writing to params is not supported"));
        }

        args.pop_front();

        let key = match args.pop_front() {
            Some(k) => k,
            None => return Err(anyhow!("param: missing key")),
        };

        let value = match args.pop_front() {
            Some(v) => v,
            None => return Err(anyhow!("param: missing value")),
        };

        Ok(Stage::new(StageKind::Input(Box::new(ParamHandler {
            key,
            value,
        }))))
    }
}

impl BaseIoHandler for ParamHandler {}

impl InputHandler for ParamHandler {
    fn read(&self, _context: &StageExecutionContext) -> Result<Value> {
        Ok(self.parse_nested_param(&self.key, &self.value))
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
        let handler = ParamHandler {
            key: "my_key".to_string(),
            value: "my_value".to_string(),
        };

        let context = StageExecutionContext::default();

        let content = handler.read(&context).unwrap();
        assert_eq!(content, json!({"my_key": "my_value"}));
    }

    #[test]
    fn test_param_read_nested() {
        let handler = ParamHandler {
            key: "a.b.c".to_string(),
            value: "val".to_string(),
        };

        let context = StageExecutionContext::default();

        let content = handler.read(&context).unwrap();
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
        let handler = ParamHandler {
            key: "a..b.c".to_string(),
            value: "val".to_string(),
        };

        let context = StageExecutionContext::default();

        let content = handler.read(&context).unwrap();
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
        let handler = ParamHandler {
            key: "a....b".to_string(),
            value: "val".to_string(),
        };

        let context = StageExecutionContext::default();

        let content = handler.read(&context).unwrap();
        assert_eq!(content, json!({"a..b": "val"}));
    }

    #[test]
    fn test_param_read_escaped_at_end() {
        let handler = ParamHandler {
            key: "a..".to_string(),
            value: "val".to_string(),
        };

        let context = StageExecutionContext::default();

        let content = handler.read(&context).unwrap();
        assert_eq!(content, json!({"a.": "val"}));
    }

    #[test]
    fn test_param_read_triple_dot() {
        let handler = ParamHandler {
            key: "a...b".to_string(),
            value: "val".to_string(),
        };

        let context = StageExecutionContext::default();

        let content = handler.read(&context).unwrap();
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
        let args = StageArgs::new_from_args(vec![
            "param".to_string(),
            "key".to_string(),
            "value".to_string(),
        ]);
        let result = ParamHandler::new_from_args(args, true);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "param: writing to params is not supported");
        }
    }

    #[test]
    fn test_param_supports() {
        assert_eq!(KIND, "param");
    }

    #[test]
    fn test_param_try_parse_args() {
        let args = StageArgs::new_from_args(vec![
            "param".to_string(),
            "foo".to_string(),
            "bar".to_string(),
        ]);
        let stage = ParamHandler::new_from_args(args, false).unwrap();
        if let StageKind::Input(_) = stage.kind {
            // ok
        } else {
            panic!("Expected Input kind");
        }
    }

    #[test]
    fn test_param_try_parse_args_missing_value() {
        let tokens = StageArgs::new_from_args(vec!["param".to_string(), "foo".to_string()]);
        let result = ParamHandler::new_from_args(tokens, false);
        if let Err(e) = result {
            assert_eq!(e.to_string(), "param: missing value");
        } else {
            panic!("Expected Error");
        }
    }
}
