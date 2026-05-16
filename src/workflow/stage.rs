use std::collections::HashMap;

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{handlers::format::get_handler_for_format, workflow::io::IoHandler};

/// Represents a configuration source or destination with associated IO and format handlers.
pub struct Stage {
    pub io_handler: Box<dyn IoHandler>,
    pub args: HashMap<String, String>,
}

impl Stage {
    pub fn new(io_handler: Box<dyn IoHandler>, args: HashMap<String, String>) -> Self {
        Self { io_handler, args }
    }

    /// Reads raw string content from this stage.
    pub fn read(&self) -> Result<String> {
        self.io_handler.read(&self.args)
    }

    /// Writes serialized content to this stage.
    pub fn write(&self, value: &Value) -> Result<()> {
        let serialized_value = match self.args.get("format") {
            Some(f) => get_handler_for_format(f)
                .ok_or_else(|| anyhow!("Format handler not found for: {}", f))?
                .serialize(value)?,
            None => value.to_string(),
        };
        self.io_handler.write(&serialized_value, &self.args)
    }

    pub fn parse(&self, content: &str) -> Result<Value> {
        match self.args.get("format") {
            Some(f) => get_handler_for_format(f)
                .ok_or_else(|| anyhow!("Format handler not found for: {}", f))?
                .parse(content),
            None => Err(anyhow!(
                "Inputs/outputs without defined formats are not supported"
            )),
        }
    }
}
