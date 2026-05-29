use std::collections::{HashMap, VecDeque};
use std::env;

use anyhow::Result;
use serde_json::{Map, Value};

use crate::{
    jinja::JinjaEngine,
    workflow::io::{BaseIoHandler, InputHandler, TryParseResult},
    workflow::stage::{Stage, StageExecutionContext, StageKind},
};

const KIND: &str = "env";

/// Handles environment variable input operations.
/// NB! This handler converts the input into dotenv format.
///     Nesting is actually handler by dotenv format handler.
#[derive(Clone)]
pub struct EnvHandler;

impl BaseIoHandler for EnvHandler {
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
            return TryParseResult::Error(anyhow::anyhow!(
                "Environment handler: writing to environment variables is not supported"
            ));
        }

        tokens.pop_front();

        let prefix = tokens.pop_front();

        let mut args = HashMap::new();
        if let Some(p) = prefix {
            args.insert("prefix".to_string(), p);
        }
        args.insert("format".to_string(), "dotenv".to_string());

        TryParseResult::Success(Stage::new(
            StageKind::Input(Box::new(self.clone())),
            args,
            jinja.clone(),
        ))
    }
}

impl InputHandler for EnvHandler {
    fn read(
        &self,
        args: &HashMap<String, String>,
        _jinja: &JinjaEngine,
        _context: &StageExecutionContext,
    ) -> Result<Value> {
        let mut res: Map<String, Value> = Map::new();
        let prefix = args.get("prefix").map(|s| s.as_str()).unwrap_or("");

        for (key, value) in env::vars() {
            if prefix.is_empty() {
                res.insert(key, Value::String(value));
            } else {
                let prefix_with_sep = format!("{}__", prefix);
                if key.starts_with(&prefix_with_sep) {
                    let stripped_key = &key[prefix_with_sep.len()..];
                    res.insert(String::from(stripped_key), Value::String(value));
                }
            }
        }
        Ok(Value::Object(res.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // We want to minimize the chance of naming collision. So random prefix here
    const ENV_VAR_PREFIX: &str = "XLYZKXPFJH_KONFG_TESTS_HANDLERS_IO_ENV";

    #[test]
    fn test_env_read_with_prefix() {
        let var_db_host = format!("{}__MYAPP__DB__HOST", ENV_VAR_PREFIX);
        let var_db_port = format!("{}__MYAPP__DB__PORT", ENV_VAR_PREFIX);
        let var_other = format!("{}__OTHERAPP__VAR", ENV_VAR_PREFIX);
        unsafe {
            env::set_var(&var_db_host, "localhost");
            env::set_var(&var_db_port, "5432");
            env::set_var(&var_other, "value");
        }

        let handler = EnvHandler;
        let mut args = HashMap::new();
        args.insert("prefix".to_string(), format!("{}__MYAPP", ENV_VAR_PREFIX));

        let jinja = JinjaEngine::new();
        let context = StageExecutionContext {
            current_config: serde_json::Value::Object(Default::default()),
        };

        let content = handler.read(&args, &jinja, &context).unwrap();
        let obj = content.as_object().unwrap();

        assert_eq!(obj.len(), 2);
        assert_eq!(
            obj.get("DB__HOST").and_then(|v| v.as_str()).unwrap(),
            "localhost"
        );
        assert_eq!(
            obj.get("DB__PORT").and_then(|v| v.as_str()).unwrap(),
            "5432"
        );
    }

    #[test]
    fn test_env_try_parse_args_output_error() {
        let handler = EnvHandler;
        let mut tokens = VecDeque::from(vec!["env".to_string()]);
        let jinja = JinjaEngine::new();
        let result = handler.try_parse_args(&mut tokens, &jinja, true);
        assert!(matches!(result, TryParseResult::Error(_)));
        if let TryParseResult::Error(e) = result {
            assert_eq!(
                e.to_string(),
                "Environment handler: writing to environment variables is not supported"
            );
        }
    }

    #[test]
    fn test_env_supports() {
        let handler = EnvHandler;
        assert!(handler.supports("env"));
        assert!(!handler.supports("stdio"));
    }
}
