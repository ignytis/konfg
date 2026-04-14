use std::collections::VecDeque;

use anyhow::{anyhow, Result};
use std::fs;

use crate::cli::IoSpec;

use super::IoHandler;

/// Handles file input/output operations.
#[derive(Clone)]
pub struct FileHandler;

impl IoHandler for FileHandler {
    fn read(&self, source: &str) -> Result<String> {
        Ok(fs::read_to_string(source)?)
    }

    fn write(&self, dest: &str, content: &str) -> Result<()> {
        fs::write(dest, content)?;
        Ok(())
    }

    fn supports(&self, kind: &str) -> bool {
        kind == "file"
    }

    fn clone_box(&self) -> Box<dyn IoHandler> {
        Box::new(self.clone())
    }

    fn try_parse_spec(&self, tokens: &mut VecDeque<String>) -> Option<Result<IoSpec>> {
        if tokens.front().map(String::as_str) != Some("file") {
            return None;
        }
        tokens.pop_front();
        let path = match tokens.pop_front() {
            Some(v) => v,
            None => return Some(Err(anyhow!("file: missing path"))),
        };
        let format = match tokens.pop_front() {
            Some(v) => v,
            None => return Some(Err(anyhow!("file: missing format after path '{path}'"))),
        };
        Some(Ok(IoSpec::File { path, format }))
    }
}
