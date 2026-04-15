use std::collections::VecDeque;

use anyhow::{anyhow, Result};
use std::io::{Read, Write};

use crate::{
    handlers::format::get_handler_for_format, handlers::io::IoHandler, types::endpoint::Endpoint,
};

/// Handles standard input/output operations.
#[derive(Clone)]
pub struct StdioHandler;

impl IoHandler for StdioHandler {
    fn read(&self, _path: Option<&str>) -> Result<String> {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    }

    fn write(&self, content: &str, _path: Option<&str>) -> Result<()> {
        std::io::stdout().write_all(content.as_bytes())?;
        Ok(())
    }

    fn supports(&self, kind: &str) -> bool {
        kind == "stdio"
    }

    fn clone_box(&self) -> Box<dyn IoHandler> {
        Box::new(self.clone())
    }

    fn try_parse_spec(&self, tokens: &mut VecDeque<String>) -> Result<Option<Endpoint>> {
        if tokens.front().map(String::as_str) != Some("stdio") {
            return Ok(None);
        }
        tokens.pop_front();
        let format = match tokens.pop_front() {
            Some(v) => v,
            None => return Err(anyhow!("stdio: missing format")),
        };

        let format_handler = match get_handler_for_format(&format) {
            Some(h) => Some(h),
            None => return Err(anyhow!("stdio handler: unknown format {}", format)),
        };

        Ok(Some(Endpoint::new(self.clone_box(), format_handler, None)))
    }
}
