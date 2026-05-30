use std::collections::VecDeque;
use std::ffi::OsStr;
use std::path::Path;

use anyhow::{Result, anyhow};

use crate::handlers::format;

/// Resolve format name either from next token (if it matches a format handler)
/// or by guessing from file extension.
/// If a format token is used, it will be consumed from `tokens`.
pub fn resolve_format_from_tokens(path: &str, tokens: &mut VecDeque<String>) -> Result<String> {
    let next_token_maybe_format = match tokens.front() {
        Some(t) => t,
        None => "",
    };

    // Try to get handler by next token
    if let Some(h) = format::get_handler_for_format(next_token_maybe_format) {
        // consume format token
        tokens.pop_front();
        return Ok(h.get_format_name().to_string());
    }

    // Guess by file extension
    let ext = Path::new(path)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("");

    match format::get_handler_for_file_extension(ext) {
        Ok(h) => Ok(h.get_format_name().to_string()),
        Err(_) => Err(anyhow!(
            "Failed to find the format handler using CLI arguments or file extension"
        )),
    }
}
