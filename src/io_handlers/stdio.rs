use anyhow::Result;
use std::io::{Read, Write};

use super::IoHandler;

/// Handles standard input/output operations.
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

    fn supports(&self, scheme: &str) -> bool {
        scheme.starts_with("stdio-")
    }
}
