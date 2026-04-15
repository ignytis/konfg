use std::collections::VecDeque;

use anyhow::{anyhow, Result};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use crate::{
    handlers::{format, io::IoHandler},
    types::endpoint::Endpoint,
};

const KIND: &str = "file";

/// Handles file input/output operations.
#[derive(Clone)]
pub struct FileHandler;

impl IoHandler for FileHandler {
    fn read(&self, path: Option<&str>) -> Result<String> {
        match path {
            Some(p) => Ok(fs::read_to_string(p)?),
            None => Err(anyhow!("File handler: path is not specified")),
        }
    }

    fn write(&self, content: &str, path: Option<&str>) -> Result<()> {
        let path = match path {
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

    fn try_parse_spec(&self, tokens: &mut VecDeque<String>) -> Result<Option<Endpoint>> {
        let is_first_token_kind_keyword = match tokens.front().map(String::as_str) {
            Some(KIND) => true,
            Some(maybe_path) => {
                if !Path::new(maybe_path).exists() {
                    return Ok(None);
                }
                false
            }
            None => return Ok(None),
        };
        if is_first_token_kind_keyword {
            tokens.pop_front();
        }

        let path = match tokens.pop_front() {
            Some(v) => v,
            None => return Err(anyhow!("file: missing path")),
        };

        // Check the next token. If it is supported format, use it. Otherwise try to guess the format from filename
        let next_token_maybe_format = match tokens.front() {
            Some(t) => t,
            None => "",
        };

        // Try to get handler by next token
        let format_handler = match format::get_handler_for_format(next_token_maybe_format) {
            Some(h) => {
                tokens.pop_front();
                h
            }
            None => {
                let ext = Path::new(path.as_str())
                    .extension()
                    .and_then(OsStr::to_str)
                    .unwrap_or("");
                match format::get_handler_for_file_extension(ext) {
                    Ok(h) => h,
                    Err(_) => {
                        return Err(anyhow!(
                        "Failed to find the format handler using CLI arguments or file extension"
                    ))
                    }
                }
            }
        };

        Ok(Some(Endpoint::new(
            self.clone_box(),
            Some(format_handler),
            Some(path),
        )))
    }
}
