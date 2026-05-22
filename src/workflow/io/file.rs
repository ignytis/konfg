use std::collections::{HashMap, VecDeque};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    handlers::format::{self, get_handler_for_format},
    jinja::JinjaEngine,
    workflow::io::{BaseIoHandler, InputHandler, OutputHandler, TryParseResult},
    workflow::stage::{Stage, StageKind},
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
        let is_first_token_kind_keyword = match tokens.front().map(String::as_str) {
            Some(KIND) => true,
            Some(maybe_path) => {
                if !Path::new(maybe_path).exists() {
                    return TryParseResult::NotSupported;
                }
                false
            }
            None => return TryParseResult::NotSupported,
        };
        if is_first_token_kind_keyword {
            tokens.pop_front();
        }

        let path = match tokens.pop_front() {
            Some(v) => v,
            None => return TryParseResult::Error(anyhow!("file: missing path")),
        };

        // Check the next token. If it is supported format, use it. Otherwise try to guess the format from filename
        let next_token_maybe_format = match tokens.front() {
            Some(t) => t,
            None => "",
        };

        // Try to get handler by next token
        let format_name: String = match format::get_handler_for_format(next_token_maybe_format) {
            Some(h) => {
                tokens.pop_front();
                h.get_format_name().to_string()
            }
            None => {
                let ext = Path::new(path.as_str())
                    .extension()
                    .and_then(OsStr::to_str)
                    .unwrap_or("");
                match format::get_handler_for_file_extension(ext) {
                    Ok(h) => h.get_format_name().to_string(),
                    Err(_) => {
                        return TryParseResult::Error(anyhow!(
                            "Failed to find the format handler using CLI arguments or file extension"
                        ));
                    }
                }
            }
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
        jinja: &JinjaEngine,
        context: &serde_json::Value,
    ) -> Result<Value> {
        let path = args
            .get("path")
            .ok_or_else(|| anyhow!("File handler: path is not specified"))?;
        let raw = fs::read_to_string(path)?;
        let rendered = jinja.render(&raw, context)?;

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

impl OutputHandler for FileHandler {
    fn write(&self, content: &str, args: &HashMap<String, String>) -> Result<()> {
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
            "file".to_string(),
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
            "file".to_string(),
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
        assert!(handler.supports("file"));
        assert!(!handler.supports("stdio"));
    }
}
