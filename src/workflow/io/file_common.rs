use std::collections::{HashMap, VecDeque};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    file_format_handlers::{self, get_handler_for_format},
    jinja::JinjaEngine,
    workflow::io::{BaseIoHandler, InputHandler, OutputHandler, TryParseResult},
    workflow::stage::{Stage, StageExecutionContext, StageKind},
};

/// Resolve format name either from next token (if it matches a format handler)
/// or by guessing from file extension.
/// If a format token is used, it will be consumed from `tokens`.
pub fn resolve_format_from_tokens(path: &str, tokens: &mut VecDeque<String>) -> Result<String> {
    let next_token_maybe_format = match tokens.front() {
        Some(t) => t,
        None => "",
    };

    // Try to get handler by next token
    if let Some(h) = file_format_handlers::get_handler_for_format(next_token_maybe_format) {
        // consume format token
        tokens.pop_front();
        return Ok(h.get_format_name().to_string());
    }

    // Guess by file extension
    let ext = Path::new(path)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("");

    match file_format_handlers::get_handler_for_file_extension(ext) {
        Ok(h) => Ok(h.get_format_name().to_string()),
        Err(_) => Err(anyhow!(
            "Failed to find the format handler using CLI arguments or file extension"
        )),
    }
}

/// Hook for preprocessing raw file content before parsing.
/// Default implementation returns the content unchanged.
pub trait FilePreprocessor: BaseIoHandler {
    fn preprocess(
        &self,
        raw: &str,
        _jinja: &JinjaEngine,
        _context: &StageExecutionContext,
    ) -> Result<String> {
        Ok(raw.to_string())
    }
}

/// Base handler for file-based I/O. Delegates preprocessing to `FilePreprocessor`.
///
/// `GUESS_PATH`: when `true`, an unknown first token is accepted as a file path if it exists on
/// disk (tplfile behaviour). When `false`, the explicit kind keyword is required (file behaviour).
pub struct FileIoHandler<H: FilePreprocessor + Clone + Send + Sync + 'static> {
    pub kind: &'static str,
    pub inner: H,
}

impl<H: FilePreprocessor + Clone + Send + Sync + 'static> FileIoHandler<H> {
    pub fn new(kind: &'static str, inner: H) -> Self {
        Self { kind, inner }
    }

    /// Shared `try_parse_args` logic.
    /// `guess_path`: accept an existing filesystem path even without the kind keyword.
    pub fn try_parse(
        &self,
        tokens: &mut VecDeque<String>,
        jinja: &JinjaEngine,
        is_output: bool,
        guess_path: bool,
    ) -> TryParseResult {
        let is_first_token_kind_keyword = match tokens.front().map(String::as_str) {
            Some(k) if k == self.kind => true,
            Some(maybe_path) if guess_path => {
                if !Path::new(maybe_path).exists() {
                    return TryParseResult::NotSupported;
                }
                false
            }
            _ => return TryParseResult::NotSupported,
        };

        if is_first_token_kind_keyword {
            tokens.pop_front();
        }

        let path = match tokens.pop_front() {
            Some(v) => v,
            None => return TryParseResult::Error(anyhow!("{}: missing path", self.kind)),
        };

        let format_name = match resolve_format_from_tokens(&path, tokens) {
            Ok(f) => f,
            Err(e) => return TryParseResult::Error(e),
        };

        let mut args = HashMap::new();
        args.insert("path".to_string(), path);
        args.insert("format".to_string(), format_name);

        let stage_kind = if is_output {
            StageKind::Output(Box::new(self.inner.clone()))
        } else {
            StageKind::Input(Box::new(self.inner.clone()))
        };

        TryParseResult::Success(Stage::new(stage_kind, args, jinja.clone()))
    }
}

/// Blanket `InputHandler` for any `FilePreprocessor`.
impl<H: FilePreprocessor + Clone + Send + Sync + 'static> InputHandler for H {
    fn read(
        &self,
        args: &HashMap<String, String>,
        jinja: &JinjaEngine,
        context: &StageExecutionContext,
    ) -> Result<Value> {
        let path = args
            .get("path")
            .ok_or_else(|| anyhow!("File handler: path is not specified"))?;
        let raw = fs::read_to_string(path)?;
        let content = self.preprocess(&raw, jinja, context)?;

        match args.get("format") {
            Some(f) => get_handler_for_format(f)
                .ok_or_else(|| anyhow!("Format handler not found for: {}", f))?
                .parse(&content),
            None => Err(anyhow!(
                "Inputs/outputs without defined formats are not supported"
            )),
        }
    }
}

/// Blanket `OutputHandler` for any `FilePreprocessor`.
impl<H: FilePreprocessor + Clone + Send + Sync + 'static> OutputHandler for H {
    fn write(
        &self,
        content: &str,
        args: &HashMap<String, String>,
        _context: &StageExecutionContext,
    ) -> Result<()> {
        let path = args
            .get("path")
            .ok_or_else(|| anyhow!("File handler: path is not specified"))?;
        fs::write(path, content)?;
        Ok(())
    }
}
