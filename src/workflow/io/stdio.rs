use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    handlers::format::get_handler_for_format,
    jinja::JinjaEngine,
    workflow::io::{BaseIoHandler, InputHandler, OutputHandler, TryParseResult},
    workflow::stage::{Stage, StageExecutionContext, StageKind},
};

const KIND: &str = "stdio";

/// Handles standard input/output operations.
#[derive(Clone)]
pub struct StdioHandler;

impl BaseIoHandler for StdioHandler {
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
        tokens.pop_front();
        let format = match tokens.pop_front() {
            Some(v) => v,
            None => return TryParseResult::Error(anyhow!("stdio: missing format")),
        };

        let mut args = HashMap::new();
        args.insert("format".to_string(), format);

        let kind = if is_output {
            StageKind::Output(Box::new(self.clone()))
        } else {
            StageKind::Input(Box::new(self.clone()))
        };

        TryParseResult::Success(Stage::new(kind, args, jinja.clone()))
    }
}

impl InputHandler for StdioHandler {
    fn read(
        &self,
        args: &HashMap<String, String>,
        jinja: &JinjaEngine,
        context: &StageExecutionContext,
    ) -> Result<Value> {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        let rendered = jinja.render(&buf, &context.current_config)?;

        match args.get("format") {
            Some(f) => get_handler_for_format(f)
                .ok_or_else(|| anyhow!("Format handler not found for: {}", f))?
                .parse(&rendered),
            None => Err(anyhow!(
                "Inputs/outputs without defined formats are not supported"
            )),
        }
    }
}

impl OutputHandler for StdioHandler {
    fn write(
        &self,
        content: &str,
        _args: &HashMap<String, String>,
        _context: &StageExecutionContext,
    ) -> Result<()> {
        std::io::stdout().write_all(content.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdio_try_parse_args_input() {
        let handler = StdioHandler;
        let mut tokens = VecDeque::from(vec!["stdio".to_string(), "json".to_string()]);
        let jinja = JinjaEngine::new();

        let result = handler.try_parse_args(&mut tokens, &jinja, false);
        if let TryParseResult::Success(stage) = result {
            assert!(matches!(stage.kind, StageKind::Input(_)));
            assert_eq!(stage.args.get("format").unwrap(), "json");
        } else {
            panic!("Expected Success");
        }
    }

    #[test]
    fn test_stdio_try_parse_args_output() {
        let handler = StdioHandler;
        let mut tokens = VecDeque::from(vec!["stdio".to_string(), "yaml".to_string()]);
        let jinja = JinjaEngine::new();

        let result = handler.try_parse_args(&mut tokens, &jinja, true);
        if let TryParseResult::Success(stage) = result {
            assert!(matches!(stage.kind, StageKind::Output(_)));
            assert_eq!(stage.args.get("format").unwrap(), "yaml");
        } else {
            panic!("Expected Success");
        }
    }

    #[test]
    fn test_stdio_supports() {
        let handler = StdioHandler;
        assert!(handler.supports(KIND));
        assert!(!handler.supports("file"));
    }
}
