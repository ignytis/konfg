use std::collections::VecDeque;
use std::io::{Read, Write};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    file_format_handlers::get_handler_for_format,
    jinja::JinjaEngine,
    workflow::io::{BaseIoHandler, InputHandler, OutputHandler},
    workflow::stage::{Stage, StageExecutionContext, StageKind},
};

pub const KIND: &str = "stdio";

/// Handles standard input/output operations.
#[derive(Clone)]
pub struct StdioHandler {
    pub format: String,
    pub jinja: JinjaEngine,
}

impl StdioHandler {
    pub fn new_from_args(
        mut tokens: VecDeque<String>,
        jinja: &JinjaEngine,
        is_output: bool,
    ) -> Result<Stage> {
        if tokens.front().map(String::as_str) != Some(KIND) {
            return Err(anyhow!("stdio handler: not supported"));
        }
        tokens.pop_front();
        let format = match tokens.pop_front() {
            Some(v) => v,
            None => return Err(anyhow!("stdio: missing format")),
        };

        let handler = StdioHandler {
            format,
            jinja: jinja.clone(),
        };

        let kind = if is_output {
            StageKind::Output(Box::new(handler))
        } else {
            StageKind::Input(Box::new(handler))
        };

        Ok(Stage::new(kind))
    }
}

impl BaseIoHandler for StdioHandler {}

impl InputHandler for StdioHandler {
    fn read(&self, context: &StageExecutionContext) -> Result<Value> {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        let rendered = self.jinja.render(&buf, &context.current_config)?;

        get_handler_for_format(&self.format)
            .ok_or_else(|| anyhow!("Format handler not found for: {}", self.format))?
            .parse(&rendered)
    }
}

impl OutputHandler for StdioHandler {
    fn write(&self, content: &str, _context: &StageExecutionContext) -> Result<()> {
        std::io::stdout().write_all(content.as_bytes())?;
        Ok(())
    }

    fn get_format(&self) -> Option<String> {
        Some(self.format.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdio_try_parse_args_input() {
        let tokens = VecDeque::from(vec!["stdio".to_string(), "json".to_string()]);
        let jinja = JinjaEngine::new();

        let stage = StdioHandler::new_from_args(tokens, &jinja, false).unwrap();
        assert!(matches!(stage.kind, StageKind::Input(_)));
    }

    #[test]
    fn test_stdio_try_parse_args_output() {
        let tokens = VecDeque::from(vec!["stdio".to_string(), "yaml".to_string()]);
        let jinja = JinjaEngine::new();

        let stage = StdioHandler::new_from_args(tokens, &jinja, true).unwrap();
        assert!(matches!(stage.kind, StageKind::Output(_)));
    }

    #[test]
    fn test_stdio_supports() {
        assert_eq!(KIND, "stdio");
    }
}
