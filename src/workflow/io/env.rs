use std::collections::{HashMap, VecDeque};
use std::env;

use anyhow::Result;
use serde_json::Value;

use crate::{
    workflow::io::{BaseIoHandler, InputHandler},
    workflow::stage::{Stage, StageArgs, StageExecutionContext, StageKind},
};

pub const KIND: &str = "env";

/// Handles environment variable input operations.
/// NB! This handler converts the input into dotenv format.
///     Nesting is actually handler by dotenv format handler.
#[derive(Clone)]
pub struct EnvHandler {
    pub prefix: Option<String>,
}

impl EnvHandler {
    pub fn new_from_args(tokens: StageArgs, is_output: bool) -> Result<Stage> {
        let mut args = VecDeque::from(tokens.args);
        if args.front().map(String::as_str) != Some(KIND) {
            return Err(anyhow::anyhow!("env handler: not supported"));
        }

        if is_output {
            return Err(anyhow::anyhow!(
                "Environment handler: writing to environment variables is not supported"
            ));
        }

        args.pop_front();

        let prefix = args.pop_front();

        Ok(Stage::new(StageKind::Input(Box::new(EnvHandler {
            prefix,
        }))))
    }
}

impl BaseIoHandler for EnvHandler {}

impl InputHandler for EnvHandler {
    fn read(&self, _context: &StageExecutionContext) -> Result<Value> {
        let mut props: HashMap<String, String> = HashMap::new();
        let prefix = self.prefix.as_deref().unwrap_or("");

        for (key, value) in env::vars() {
            if prefix.is_empty() {
                props.insert(key.to_lowercase(), value);
            } else {
                let prefix_with_sep = format!("{}__", prefix);
                if key.starts_with(&prefix_with_sep) {
                    let stripped_key = &key[prefix_with_sep.len()..];
                    props.insert(stripped_key.to_lowercase(), value);
                }
            }
        }
        Ok(crate::utils::hashmap::hashmap_new_from_flat_hashmap(
            props, "__",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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

        let handler = EnvHandler {
            prefix: Some(format!("{}__MYAPP", ENV_VAR_PREFIX)),
        };

        let context = StageExecutionContext::default();

        let content = handler.read(&context).unwrap();
        assert_eq!(
            content,
            json!({
                "db": {
                    "host": "localhost",
                    "port": "5432"
                }
            })
        );
    }

    #[test]
    fn test_env_try_parse_args_output_error() {
        let tokens = StageArgs::new_from_args(VecDeque::from(vec!["env".to_string()]));
        let result = EnvHandler::new_from_args(tokens, true);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(
                e.to_string(),
                "Environment handler: writing to environment variables is not supported"
            );
        }
    }

    #[test]
    fn test_env_supports() {
        assert_eq!(KIND, "env");
    }
}
