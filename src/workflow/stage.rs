use std::collections::{HashMap, VecDeque};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    handlers::format::get_handler_for_format,
    workflow::io::{IoHandler, REGISTERED_HANDLERS, TryParseResult},
};

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

    /// Parses a flat list of arguments into a `Stage` using registered handlers.
    /// `tokens` is a VecDeque of string parameters for single input / output.
    /// Example: ['file', '/path/to/file.cfg', 'yaml']
    pub fn try_from_strings(mut tokens: VecDeque<String>) -> Result<Stage> {
        for io_handler in REGISTERED_HANDLERS.iter() {
            match io_handler.try_parse_args(&mut tokens) {
                TryParseResult::Success(s) => return Ok(s),
                TryParseResult::NotSupported => continue,
                TryParseResult::Error(e) => return Err(e),
            }
        }

        return Err(anyhow!("Unrecognized input argument: {:?}", tokens));
    }
}
