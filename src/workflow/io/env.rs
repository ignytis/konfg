use std::collections::{HashMap, VecDeque};
use std::env;

use anyhow::{anyhow, Result};

use crate::handlers::format::dotenv::DotenvHandler;
use crate::workflow::io::{IoHandler, Stage, TryParseResult};

const KIND: &str = "env";

/// Handles environment variable input operations.
/// NB! This handler converts the input into dotenv format.
///     Nesting is actually handler by dotenv format handler.
#[derive(Clone)]
pub struct EnvHandler;

impl IoHandler for EnvHandler {
    fn read(&self, args: &HashMap<String, String>) -> Result<String> {
        let mut res = String::new();
        let prefix = args.get("prefix").map(|s| s.as_str()).unwrap_or("");

        for (key, value) in env::vars() {
            if prefix.is_empty() {
                res.push_str(&format!("{}={}\n", key, value));
            } else {
                let prefix_with_sep = format!("{}__", prefix);
                if key.starts_with(&prefix_with_sep) {
                    let stripped_key = &key[prefix_with_sep.len()..];
                    res.push_str(&format!("{}={}\n", stripped_key, value));
                }
            }
        }
        Ok(res)
    }

    fn write(&self, _content: &str, _args: &HashMap<String, String>) -> Result<()> {
        Err(anyhow!(
            "Environment handler: writing to environment variables is not supported"
        ))
    }

    fn supports(&self, kind: &str) -> bool {
        kind == KIND
    }

    fn clone_box(&self) -> Box<dyn IoHandler> {
        Box::new(self.clone())
    }

    fn try_parse_tokens(&self, tokens: &mut VecDeque<String>) -> TryParseResult {
        if tokens.front().map(String::as_str) != Some(KIND) {
            return TryParseResult::NotSupported;
        }
        tokens.pop_front();

        let prefix = tokens.pop_front();

        let mut args = HashMap::new();
        if let Some(p) = prefix {
            args.insert("prefix".to_string(), p);
        }

        TryParseResult::Success(Stage::new(
            self.clone_box(),
            Some(Box::new(DotenvHandler)),
            args,
        ))
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

        let content = handler.read(&args).unwrap();
        let content_lines: Vec<&str> = content.trim().split("\n").collect();

        assert!(content_lines.len() == 2);
        assert!(content_lines.contains(&"DB__HOST=localhost"));
        assert!(content_lines.contains(&"DB__PORT=5432"));
    }

    #[test]
    fn test_env_write_error() {
        let handler = EnvHandler;
        let result = handler.write("content", &HashMap::new());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Environment handler: writing to environment variables is not supported"
        );
    }

    #[test]
    fn test_env_supports() {
        let handler = EnvHandler;
        assert!(handler.supports("env"));
        assert!(!handler.supports("stdio"));
    }
}
