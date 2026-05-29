use std::collections::{HashMap, VecDeque};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    handlers::format::get_handler_for_format,
    jinja::JinjaEngine,
    workflow::{
        filters::{Filter, REGISTERED_FILTERS, TryParseFilterResult},
        io::{
            InputHandler, OutputHandler, REGISTERED_HANDLERS, TryParseResult, stdio::StdioHandler,
        },
    },
};

pub enum StageKind {
    Input(Box<dyn InputHandler>),
    Output(Box<dyn OutputHandler>),
    Filter(Box<dyn Filter>),
}

/// Execution context for a stage.
pub struct StageExecutionContext {
    pub current_config: Value,
}

impl StageExecutionContext {
    pub fn new() -> Self {
        Self {
            current_config: Value::Object(Default::default()),
        }
    }
}

/// Represents a configuration source or destination with associated IO and format handlers.
pub struct Stage {
    pub kind: StageKind,
    pub args: HashMap<String, String>,
    pub jinja_engine: JinjaEngine,
}

/// A definition of stage parser function
pub type StageParserFn = fn(VecDeque<String>, &JinjaEngine) -> Result<Stage>;

impl Stage {
    pub fn new(kind: StageKind, args: HashMap<String, String>, jinja_engine: JinjaEngine) -> Self {
        Self {
            kind,
            args,
            jinja_engine,
        }
    }

    pub fn new_output_default() -> Self {
        Stage {
            kind: StageKind::Output(Box::new(StdioHandler {})),
            args: HashMap::default(),
            jinja_engine: JinjaEngine::new(),
        }
    }

    /// Returns true if the stage is an input stage.
    pub fn is_input(&self) -> bool {
        matches!(self.kind, StageKind::Input(_))
    }

    /// Returns true if the stage is an output stage.
    pub fn is_output(&self) -> bool {
        matches!(self.kind, StageKind::Output(_))
    }

    /// Returns true if the stage is a filter stage.
    pub fn is_filter(&self) -> bool {
        matches!(self.kind, StageKind::Filter(_))
    }

    /// Executes the stage: reads content for input stages, writes content for output stages, or applies filter for filter stages.
    pub fn run(&self, context: &mut StageExecutionContext) -> Result<Value> {
        match &self.kind {
            StageKind::Input(handler) => handler.read(&self.args, &self.jinja_engine, context),
            StageKind::Output(handler) => {
                let serialized_value = match self.args.get("format") {
                    Some(f) => get_handler_for_format(f)
                        .ok_or_else(|| anyhow!("Format handler not found for: {}", f))?
                        .serialize(&context.current_config)?,
                    None => context.current_config.to_string(),
                };
                handler.write(&serialized_value, &self.args, context)?;
                Ok(Value::Null)
            }
            StageKind::Filter(handler) => {
                handler.apply(&self.args, context)?;
                Ok(Value::Null)
            }
        }
    }

    /// Parses a flat list of arguments into a `Stage` using registered handlers.
    /// `tokens` is a VecDeque of string parameters for single input.
    /// Example: ['file', '/path/to/file.cfg', 'yaml']
    pub fn try_from_strings_input(
        mut tokens: VecDeque<String>,
        jinja: &JinjaEngine,
    ) -> Result<Stage> {
        for io_handler in REGISTERED_HANDLERS.iter() {
            match io_handler.try_parse_args(&mut tokens, &jinja, false) {
                TryParseResult::Success(s) => return Ok(s),
                TryParseResult::NotSupported => continue,
                TryParseResult::Error(e) => return Err(e),
            }
        }

        return Err(anyhow!("Unrecognized input argument: {:?}", tokens));
    }

    /// Parses a flat list of arguments into a `Stage` using registered handlers.
    /// `tokens` is a VecDeque of string parameters for single output.
    /// Example: ['file', '/path/to/file.cfg', 'yaml']
    pub fn try_from_strings_output(
        mut tokens: VecDeque<String>,
        jinja: &JinjaEngine,
    ) -> Result<Stage> {
        for io_handler in REGISTERED_HANDLERS.iter() {
            match io_handler.try_parse_args(&mut tokens, &jinja, true) {
                TryParseResult::Success(s) => return Ok(s),
                TryParseResult::NotSupported => continue,
                TryParseResult::Error(e) => return Err(e),
            }
        }

        return Err(anyhow!("Unrecognized input argument: {:?}", tokens));
    }

    /// Parses a flat list of arguments into a `Stage` using registered filter handlers.
    pub fn try_from_strings_filter(
        mut tokens: VecDeque<String>,
        jinja: &JinjaEngine,
    ) -> Result<Stage> {
        for filter_handler in REGISTERED_FILTERS.iter() {
            match filter_handler.try_parse_args(&mut tokens, &jinja) {
                TryParseFilterResult::Success(s) => return Ok(s),
                TryParseFilterResult::NotSupported => continue,
                TryParseFilterResult::Error(e) => return Err(e),
            }
        }

        return Err(anyhow!("Unrecognized filter argument: {:?}", tokens));
    }
}
