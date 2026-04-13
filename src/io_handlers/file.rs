use anyhow::Result;
use std::fs;

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

    fn supports(&self, scheme: &str) -> bool {
        scheme.starts_with("file-")
    }

    fn clone_box(&self) -> Box<dyn IoHandler> {
        Box::new(self.clone())
    }
}
