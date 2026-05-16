use std::collections::{HashMap, VecDeque};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{Result, anyhow};

use crate::handlers::format;
use crate::jinja::JinjaEngine;
use crate::workflow::io::{IoHandler, Stage, TryParseResult};

const KIND: &str = "file";

/// Handles file input/output operations.
#[derive(Clone)]
pub struct FileHandler;

impl IoHandler for FileHandler {
    fn read(
        &self,
        args: &HashMap<String, String>,
        jinja: &JinjaEngine,
        context: &serde_json::Value,
    ) -> Result<String> {
        let path = args
            .get("path")
            .ok_or_else(|| anyhow!("File handler: path is not specified"))?;
        let raw = fs::read_to_string(path)?;
        let rendered = jinja.render(&raw, context)?;
        Ok(rendered)
    }

    fn write(&self, content: &str, args: &HashMap<String, String>) -> Result<()> {
        let path = match args.get("path") {
            Some(p) => p,
            None => return Err(anyhow!("File handler: path is not specified")),
        };
        fs::write(path, content)?;
        Ok(())
    }

    fn supports(&self, kind: &str) -> bool {
        kind == KIND
    }

    fn clone_box(&self) -> Box<dyn IoHandler> {
        Box::new(self.clone())
    }

    fn try_parse_args(&self, tokens: &mut VecDeque<String>, jinja: &JinjaEngine) -> TryParseResult {
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

        TryParseResult::Success(Stage::new(self.clone_box(), args, jinja.clone()))
    }
}
