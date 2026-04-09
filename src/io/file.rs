use anyhow::Result;
use std::fs;

use super::IoHandler;

/// Handles file input/output operations.
pub struct FileHandler;

impl IoHandler for FileHandler {
    fn read(&self, source: &str) -> Result<String> {
        Ok(fs::read_to_string(source)?)
    }

    fn write(&self, dest: &str, content: &str) -> Result<()> {
        fs::write(dest, content)?;
        Ok(())
    }

    fn supports(&self, io_kind: &str) -> bool {
        // Supports "file" scheme or any path that doesn't start with stdin/stdout
        io_kind.starts_with("file") || (!io_kind.starts_with("stdin") && !io_kind.starts_with("stdout"))
    }
}
