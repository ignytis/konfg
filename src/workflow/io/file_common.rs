use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    file_format_handlers::{get_handler_for_file_extension, get_handler_for_format},
    workflow::io::{BaseIoHandler, InputHandler, OutputHandler},
    workflow::stage::StageExecutionContext,
};

/// Resolve format name either from next token (if it matches a format handler)
/// or by guessing from file extension.
/// Returns `(format_name, tokens_consumed)` where `tokens_consumed` is 1 if a format
/// token was used, or 0 if the format was inferred from the file extension.
pub fn resolve_format_from_tokens(path: &str, tokens: &[String]) -> Result<(String, usize)> {
    let next_token_maybe_format = tokens.first().map(String::as_str).unwrap_or("");

    // Try to get handler by next token
    if get_handler_for_format(next_token_maybe_format).is_some() {
        return Ok((next_token_maybe_format.to_string(), 1));
    }

    // Guess by file extension
    let ext = Path::new(path)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("");

    match get_handler_for_file_extension(ext) {
        Ok((id, _)) => Ok((id.to_string(), 0)),
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
        _path: &str,
        _context: &StageExecutionContext,
    ) -> Result<String> {
        Ok(raw.to_string())
    }

    fn get_path(&self) -> &str;
    fn get_format(&self) -> &str;
}

/// Blanket `InputHandler` for any `FilePreprocessor`.
impl<H: FilePreprocessor + Clone + Send + Sync + 'static> InputHandler for H {
    fn read(&self, context: &StageExecutionContext) -> Result<Value> {
        let path = self.get_path();
        let raw = fs::read_to_string(path)?;
        let content = self.preprocess(&raw, &path, context)?;

        let format = self.get_format();
        get_handler_for_format(format)
            .ok_or_else(|| anyhow!("Format handler not found for: {}", format))?
            .parse(&content)
    }
}

/// Blanket `OutputHandler` for any `FilePreprocessor`.
impl<H: FilePreprocessor + Clone + Send + Sync + 'static> OutputHandler for H {
    fn write(&self, content: &str, _context: &StageExecutionContext) -> Result<()> {
        let path = self.get_path();
        fs::write(path, content)?;
        Ok(())
    }

    fn get_format(&self) -> Option<String> {
        Some(self.get_format().to_string())
    }
}
