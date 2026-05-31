use std::collections::{HashMap, VecDeque};
use std::fs;

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    file_format_handlers::get_handler_for_format,
    jinja::JinjaEngine,
    workflow::io::{BaseIoHandler, InputHandler, OutputHandler, TryParseResult},
    workflow::stage::{Stage, StageExecutionContext, StageKind},
};

const KIND: &str = "file";

/// Handles file input/output operations.
#[derive(Clone)]
pub struct FileHandler;

impl BaseIoHandler for FileHandler {
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
        // File handler requires explicit "file" keyword. Do not attempt to guess paths here.
        match tokens.front().map(String::as_str) {
            Some(KIND) => {
                tokens.pop_front();
            }
            _ => return TryParseResult::NotSupported,
        }

        let path = match tokens.pop_front() {
            Some(v) => v,
            None => return TryParseResult::Error(anyhow!("file: missing path")),
        };

        // Resolve format using shared helper
        let format_name =
            match crate::workflow::io::file_common::resolve_format_from_tokens(&path, tokens) {
                Ok(f) => f,
                Err(e) => return TryParseResult::Error(e),
            };

        let mut args = HashMap::new();
        args.insert("path".to_string(), path);
        args.insert("format".to_string(), format_name);

        let kind = if is_output {
            StageKind::Output(Box::new(self.clone()))
        } else {
            StageKind::Input(Box::new(self.clone()))
        };

        TryParseResult::Success(Stage::new(kind, args, jinja.clone()))
    }
}

impl InputHandler for FileHandler {
    fn read(
        &self,
        args: &HashMap<String, String>,
        _jinja: &JinjaEngine,
        _context: &StageExecutionContext,
    ) -> Result<Value> {
        let path = args
            .get("path")
            .ok_or_else(|| anyhow!("File handler: path is not specified"))?;
        let raw = fs::read_to_string(path)?;

        match args.get("format") {
            Some(f) => get_handler_for_format(f)
                .ok_or_else(|| anyhow!("Format handler not found for: {}", f))?
                .parse(&raw),
            None => Err(anyhow!(
                "Inputs/outputs without defined formats are not supported"
            )),
        }
    }
}

impl OutputHandler for FileHandler {
    fn write(
        &self,
        content: &str,
        args: &HashMap<String, String>,
        _context: &StageExecutionContext,
    ) -> Result<()> {
        let path = match args.get("path") {
            Some(p) => p,
            None => return Err(anyhow!("File handler: path is not specified")),
        };
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_try_parse_args_input() {
        let handler = FileHandler;
        let mut tokens = VecDeque::from(vec![
            KIND.to_string(),
            "non_existent_file.yaml".to_string(),
            "yaml".to_string(),
        ]);
        let jinja = JinjaEngine::new();

        let result = handler.try_parse_args(&mut tokens, &jinja, false);
        if let TryParseResult::Success(stage) = result {
            assert!(matches!(stage.kind, StageKind::Input(_)));
            assert_eq!(stage.args.get("path").unwrap(), "non_existent_file.yaml");
            assert_eq!(stage.args.get("format").unwrap(), "yaml");
        } else {
            panic!("Expected Success");
        }
    }

    #[test]
    fn test_file_try_parse_args_output() {
        let handler = FileHandler;
        let mut tokens = VecDeque::from(vec![
            KIND.to_string(),
            "output.json".to_string(),
            "json".to_string(),
        ]);
        let jinja = JinjaEngine::new();

        let result = handler.try_parse_args(&mut tokens, &jinja, true);
        if let TryParseResult::Success(stage) = result {
            assert!(matches!(stage.kind, StageKind::Output(_)));
            assert_eq!(stage.args.get("path").unwrap(), "output.json");
            assert_eq!(stage.args.get("format").unwrap(), "json");
        } else {
            panic!("Expected Success");
        }
    }

    #[test]
    fn test_file_supports() {
        let handler = FileHandler;
        assert!(handler.supports(KIND));
        assert!(!handler.supports("stdio"));
    }
}
