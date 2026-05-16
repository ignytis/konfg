use std::collections::{HashMap, VecDeque};

use anyhow::{Result, anyhow};
use serde_json::{Map, Value};

use crate::{
    jinja::JinjaEngine,
    workflow::io::{IoHandler, Stage, TryParseResult},
};

const KIND: &str = "param";

/// Handles single key-value pair input operations.
#[derive(Clone)]
pub struct ParamHandler;

impl IoHandler for ParamHandler {
    fn read(
        &self,
        args: &HashMap<String, String>,
        _jinja: &JinjaEngine,
        _context: &serde_json::Value,
    ) -> Result<Value> {
        let key = args
            .get("key")
            .ok_or_else(|| anyhow!("Param handler: key is not specified"))?;
        let value = args
            .get("value")
            .ok_or_else(|| anyhow!("Param handler: value is not specified"))?;

        let mut res = Map::new();
        res.insert(key.clone(), Value::String(value.clone()));

        Ok(Value::Object(res))
    }

    fn write(&self, _content: &str, _args: &HashMap<String, String>) -> Result<()> {
        Err(anyhow!("Param handler: writing to params is not supported"))
    }

    fn supports(&self, kind: &str) -> bool {
        kind == KIND
    }

    fn clone_box(&self) -> Box<dyn IoHandler> {
        Box::new(self.clone())
    }

    fn try_parse_args(&self, tokens: &mut VecDeque<String>, jinja: &JinjaEngine) -> TryParseResult {
        if tokens.front().map(String::as_str) != Some(KIND) {
            return TryParseResult::NotSupported;
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

        TryParseResult::Success(Stage::new(self.clone_box(), args, jinja.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_read() {
        let handler = ParamHandler;
        let mut args = HashMap::new();
        args.insert("key".to_string(), "my_key".to_string());
        args.insert("value".to_string(), "my_value".to_string());

        let jinja = JinjaEngine::new();
        let context = Value::Object(Default::default());

        let content = handler.read(&args, &jinja, &context).unwrap();
        let obj = content.as_object().unwrap();

        assert_eq!(obj.len(), 1);
        assert_eq!(
            obj.get("my_key").and_then(|v| v.as_str()).unwrap(),
            "my_value"
        );
    }

    #[test]
    fn test_param_write_error() {
        let handler = ParamHandler;
        let result = handler.write("content", &HashMap::new());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Param handler: writing to params is not supported"
        );
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

        let result = handler.try_parse_args(&mut tokens, &jinja);
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

        let result = handler.try_parse_args(&mut tokens, &jinja);
        if let TryParseResult::Error(e) = result {
            assert_eq!(e.to_string(), "param: missing value");
        } else {
            panic!("Expected Error");
        }
    }
}
