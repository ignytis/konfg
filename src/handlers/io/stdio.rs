use std::collections::VecDeque;

use anyhow::{anyhow, Result};
use std::io::{Read, Write};

use crate::cli::IoSpec;

use super::IoHandler;

/// Handles standard input/output operations.
#[derive(Clone)]
pub struct StdioHandler;

impl IoHandler for StdioHandler {
    fn read(&self, _source: &str) -> Result<String> {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    }

    fn write(&self, _dest: &str, content: &str) -> Result<()> {
        std::io::stdout().write_all(content.as_bytes())?;
        Ok(())
    }

    fn supports(&self, kind: &str) -> bool {
        kind == "stdio"
    }

    fn clone_box(&self) -> Box<dyn IoHandler> {
        Box::new(self.clone())
    }

    fn try_parse_spec(&self, tokens: &mut VecDeque<String>) -> Option<Result<IoSpec>> {
        if tokens.front().map(String::as_str) != Some("stdio") {
            return None;
        }
        tokens.pop_front();
        let format = match tokens.pop_front() {
            Some(v) => v,
            None => return Some(Err(anyhow!("stdio: missing format"))),
        };
        Some(Ok(IoSpec::Stdio { format }))
    }
}
