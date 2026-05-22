use std::collections::{HashMap, VecDeque};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    handlers::format::get_handler_for_format,
    jinja::JinjaEngine,
    workflow::io::{InputHandler, OutputHandler, REGISTERED_HANDLERS, TryParseResult},
};

pub enum StageKind {
    Input(Box<dyn InputHandler>),
    Output(Box<dyn OutputHandler>),
}

/// Represents a configuration source or destination with associated IO and format handlers.
pub struct Stage {
    pub kind: StageKind,
    pub args: HashMap<String, String>,
    pub jinja_engine: JinjaEngine,
}

impl Stage {
    pub fn new(kind: StageKind, args: HashMap<String, String>, jinja_engine: JinjaEngine) -> Self {
        Self {
            kind,
            args,
            jinja_engine,
        }
    }

    /// Executes the stage: reads content for input stages, or writes content for output stages.
    pub fn run(&self, value: &Value) -> Result<Value> {
        match &self.kind {
            StageKind::Input(handler) => handler.read(&self.args, &self.jinja_engine, value),
            StageKind::Output(handler) => {
                let serialized_value = match self.args.get("format") {
                    Some(f) => get_handler_for_format(f)
                        .ok_or_else(|| anyhow!("Format handler not found for: {}", f))?
                        .serialize(value)?,
                    None => value.to_string(),
                };
                handler.write(&serialized_value, &self.args)?;
                Ok(Value::Null)
            }
        }
    }

    /// Parses a flat list of arguments into a `Stage` using registered handlers.
    /// `tokens` is a VecDeque of string parameters for single input / output.
    /// Example: ['file', '/path/to/file.cfg', 'yaml']
    pub fn try_from_strings(
        mut tokens: VecDeque<String>,
        jinja: JinjaEngine,
        is_output: bool,
    ) -> Result<Stage> {
        for io_handler in REGISTERED_HANDLERS.iter() {
            match io_handler.try_parse_args(&mut tokens, &jinja, is_output) {
                TryParseResult::Success(s) => return Ok(s),
                TryParseResult::NotSupported => continue,
                TryParseResult::Error(e) => return Err(e),
            }
        }

        return Err(anyhow!("Unrecognized input argument: {:?}", tokens));
    }
}
