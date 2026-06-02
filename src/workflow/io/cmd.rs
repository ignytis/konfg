use std::collections::VecDeque;
use std::process::Command;

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    file_format_handlers::get_handler_for_format,
    jinja::JinjaEngine,
    workflow::io::{BaseIoHandler, InputHandler},
    workflow::stage::{Stage, StageExecutionContext, StageKind},
};

pub const KIND: &str = "cmd";

/// Handles command execution input operations.
#[derive(Clone)]
pub struct CmdHandler {
    pub command: String,
    pub format: String,
}

impl CmdHandler {
    pub fn new_from_args(
        mut tokens: VecDeque<String>,
        _jinja: &JinjaEngine,
        is_output: bool,
    ) -> Result<Stage> {
        if tokens.front().map(String::as_str) != Some(KIND) {
            return Err(anyhow!("cmd handler: not supported"));
        }

        if is_output {
            return Err(anyhow!(
                "Command handler: writing to command is not supported"
            ));
        }

        tokens.pop_front();

        let format = match tokens.pop_front() {
            Some(v) => v,
            None => return Err(anyhow!("cmd: missing format")),
        };
        let command = tokens
            .iter()
            .map(|item| item.as_str())
            .collect::<Vec<&str>>()
            .join(" ");
        tokens.clear();

        Ok(Stage::new(StageKind::Input(Box::new(CmdHandler {
            command,
            format,
        }))))
    }
}

impl BaseIoHandler for CmdHandler {}

impl InputHandler for CmdHandler {
    fn read(&self, _context: &StageExecutionContext) -> Result<Value> {
        let command_str = &self.command;

        #[cfg(windows)]
        let mut cmd = Command::new("cmd");
        #[cfg(windows)]
        cmd.arg("/C").arg(command_str);

        #[cfg(not(windows))]
        let mut cmd = Command::new("sh");
        #[cfg(not(windows))]
        cmd.arg("-c").arg(command_str);

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Command failed with exit code {}: {}",
                output.status.code().unwrap_or(-1),
                stderr
            ));
        }

        let stdout = String::from_utf8(output.stdout)?;

        get_handler_for_format(&self.format)
            .ok_or_else(|| anyhow!("Format handler not found for: {}", self.format))?
            .parse(&stdout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cmd_read_echo_json() {
        let handler = CmdHandler {
            command: if cfg!(windows) {
                "echo {\"foo\": \"bar\"}".to_string()
            } else {
                "echo '{\"foo\": \"bar\"}'".to_string()
            },
            format: "json".to_string(),
        };

        let context = StageExecutionContext::default();

        let content = handler.read(&context).unwrap();
        assert_eq!(content, json!({"foo": "bar"}));
    }

    #[test]
    fn test_cmd_read_fail() {
        let handler = CmdHandler {
            command: "false".to_string(),
            format: "json".to_string(),
        };

        let context = StageExecutionContext::default();

        let result = handler.read(&context);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_supports() {
        assert_eq!(KIND, "cmd");
    }

    #[test]
    fn test_cmd_try_parse_args() {
        let tokens = VecDeque::from(vec![
            "cmd".to_string(),
            "json".to_string(),
            "ls".to_string(),
        ]);
        let jinja = JinjaEngine::new();

        let stage = CmdHandler::new_from_args(tokens, &jinja, false).unwrap();
        if let StageKind::Input(_) = stage.kind {
            // ok
        } else {
            panic!("Expected Input kind");
        }
    }
}
