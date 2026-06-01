use std::collections::{HashMap, VecDeque};
use std::process::Command;

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    file_format_handlers::get_handler_for_format,
    jinja::JinjaEngine,
    workflow::io::{BaseIoHandler, InputHandler, TryParseResult},
    workflow::stage::{Stage, StageExecutionContext, StageKind},
};

const KIND: &str = "cmd";

/// Handles command execution input operations.
#[derive(Clone)]
pub struct CmdHandler;

impl BaseIoHandler for CmdHandler {
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
            return TryParseResult::Error(anyhow!(
                "Command handler: writing to command is not supported"
            ));
        }

        tokens.pop_front();

        let format = match tokens.pop_front() {
            Some(v) => v,
            None => return TryParseResult::Error(anyhow!("cmd: missing format")),
        };
        let command = tokens
            .iter()
            .map(|item| item.as_str())
            .collect::<Vec<&str>>()
            .join(" ");
        tokens.clear();

        let mut args = HashMap::new();
        args.insert("command".to_string(), command);
        args.insert("format".to_string(), format);

        TryParseResult::Success(Stage::new(
            StageKind::Input(Box::new(self.clone())),
            args,
            jinja.clone(),
        ))
    }
}

impl InputHandler for CmdHandler {
    fn read(
        &self,
        args: &HashMap<String, String>,
        _jinja: &JinjaEngine,
        _context: &StageExecutionContext,
    ) -> Result<Value> {
        let command_str = args
            .get("command")
            .ok_or_else(|| anyhow!("Command handler: command is not specified"))?;

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

        match args.get("format") {
            Some(f) => get_handler_for_format(f)
                .ok_or_else(|| anyhow!("Format handler not found for: {}", f))?
                .parse(&stdout),
            None => Err(anyhow!(
                "Inputs/outputs without defined formats are not supported"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cmd_read_echo_json() {
        let handler = CmdHandler;
        let mut args = HashMap::new();
        #[cfg(not(windows))]
        args.insert(
            "command".to_string(),
            "echo '{\"foo\": \"bar\"}'".to_string(),
        );
        #[cfg(windows)]
        args.insert("command".to_string(), "echo {\"foo\": \"bar\"}".to_string());

        args.insert("format".to_string(), "json".to_string());

        let jinja = JinjaEngine::new();
        let context = StageExecutionContext::default();

        let content = handler.read(&args, &jinja, &context).unwrap();
        assert_eq!(content, json!({"foo": "bar"}));
    }

    #[test]
    fn test_cmd_read_fail() {
        let handler = CmdHandler;
        let mut args = HashMap::new();
        args.insert("command".to_string(), "false".to_string());
        args.insert("format".to_string(), "json".to_string());

        let jinja = JinjaEngine::new();
        let context = StageExecutionContext::default();

        let result = handler.read(&args, &jinja, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_supports() {
        let handler = CmdHandler;
        assert!(handler.supports(KIND));
        assert!(!handler.supports("stdio"));
    }

    #[test]
    fn test_cmd_try_parse_args() {
        let handler = CmdHandler;
        let mut tokens = VecDeque::from(vec![
            "cmd".to_string(),
            "ls".to_string(),
            "json".to_string(),
        ]);
        let jinja = JinjaEngine::new();

        let result = handler.try_parse_args(&mut tokens, &jinja, false);
        if let TryParseResult::Success(stage) = result {
            assert_eq!(stage.args.get("command").unwrap(), "ls");
            assert_eq!(stage.args.get("format").unwrap(), "json");
        } else {
            panic!("Expected Success");
        }
        assert!(tokens.is_empty());
    }
}
