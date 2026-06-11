use std::collections::{HashMap, VecDeque};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    file_format_handlers::get_handler_for_format,
    jinja::JinjaEngine,
    workflow::{
        filters::{Filter, REGISTERED_FILTERS},
        io::{InputHandler, OutputHandler, REGISTERED_HANDLERS, stdio::StdioHandler},
    },
};

/// The kinds of stages supported by the workflow.
pub enum StageKind {
    /// An input stage that reads configuration.
    Input(Box<dyn InputHandler>),
    /// An output stage that writes configuration.
    Output(Box<dyn OutputHandler>),
    /// A filter stage that modifies configuration.
    Filter(Box<dyn Filter>),
    /// A stage that modifies the active merge strategy.
    MergeStrategy {
        /// The path where the strategy is applied.
        path: String,
        /// The strategy ID followed by strategy-specific arguments.
        strategy: VecDeque<String>,
    },
}

/// Execution context for a stage.
#[derive(Default)]
pub struct StageExecutionContext {
    pub current_config: Value,
    pub stash: HashMap<String, Value>,
    pub merge_strategies: HashMap<String, VecDeque<String>>,
}

impl StageExecutionContext {
    pub fn new() -> Self {
        Self {
            current_config: Value::Object(Default::default()),
            stash: HashMap::new(),
            merge_strategies: HashMap::new(),
        }
    }
}

/// Represents a configuration source or destination with associated IO and format handlers.
pub struct Stage {
    pub kind: StageKind,
}

impl Stage {
    pub fn new(kind: StageKind) -> Self {
        Self { kind }
    }

    pub fn new_output_default() -> Self {
        Stage {
            kind: StageKind::Output(Box::new(StdioHandler {
                format: "json".to_string(),
                jinja: JinjaEngine::new(),
            })),
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

    /// Executes the stage: reads content for input stages, writes content for output stages, or applies filter for filter stages.
    pub fn run(&self, context: &mut StageExecutionContext) -> Result<Value> {
        match &self.kind {
            StageKind::Input(handler) => handler.read(context),
            StageKind::Output(handler) => {
                let serialized_value = match handler.get_format() {
                    Some(f) => get_handler_for_format(&f)
                        .ok_or_else(|| anyhow!("Format handler not found for: {}", f))?
                        .serialize(&context.current_config)?,
                    None => context.current_config.to_string(),
                };
                handler.write(&serialized_value, context)?;
                Ok(Value::Null)
            }
            StageKind::Filter(handler) => {
                handler.apply(context)?;
                Ok(Value::Null)
            }
            StageKind::MergeStrategy { path, strategy } => {
                context
                    .merge_strategies
                    .insert(path.clone(), strategy.clone());
                Ok(Value::Null)
            }
        }
    }

    /// Parses a flat list of arguments into a `Stage` using registered handlers.
    /// `tokens` is a VecDeque of string parameters for single input.
    /// Example: ['file', '/path/to/file.cfg', 'yaml']
    pub fn try_from_strings_input(tokens: VecDeque<String>, jinja: &JinjaEngine) -> Result<Stage> {
        let id = match tokens.front().map(String::as_str) {
            Some(i) => i,
            None => return Err(anyhow!("Missing input id")),
        };

        for (it_id, it_creator_fn) in REGISTERED_HANDLERS.iter() {
            if !id.eq(*it_id) {
                continue;
            }

            match it_creator_fn(tokens.clone(), jinja, false) {
                Ok(s) => return Ok(s),
                Err(_) if !id.eq(*it_id) => continue, // If we were guessing, ignore error and try next
                Err(e) => return Err(e),
            }
        }

        return Err(anyhow!("Unrecognized input argument: {:?}", tokens));
    }

    /// Parses a flat list of arguments into a `Stage` using registered handlers.
    /// `tokens` is a VecDeque of string parameters for single output.
    /// Example: ['file', '/path/to/file.cfg', 'yaml']
    pub fn try_from_strings_output(tokens: VecDeque<String>, jinja: &JinjaEngine) -> Result<Stage> {
        let id = match tokens.front().map(String::as_str) {
            Some(i) => i,
            None => return Err(anyhow!("Missing output id")),
        };

        for (it_id, it_creator_fn) in REGISTERED_HANDLERS.iter() {
            if !id.eq(*it_id) {
                if *it_id != "tplfile" && *it_id != "file" {
                    continue;
                }
            }

            match it_creator_fn(tokens.clone(), jinja, true) {
                Ok(s) => return Ok(s),
                Err(_) if !id.eq(*it_id) => continue,
                Err(e) => return Err(e),
            }
        }

        return Err(anyhow!("Unrecognized output argument: {:?}", tokens));
    }

    /// Parses a flat list of arguments into a `Stage` using registered filter handlers.
    pub fn try_from_strings_filter(
        mut tokens: VecDeque<String>,
        _jinja: &JinjaEngine,
    ) -> Result<Stage> {
        let id = match tokens.pop_front() {
            Some(i) => i,
            None => return Err(anyhow!("Missing filter id")),
        };

        for (it_id, it_creator_fn) in REGISTERED_FILTERS.iter() {
            if !id.eq(it_id) {
                continue;
            }

            let handler = it_creator_fn(tokens)?;
            return Ok(Stage {
                kind: StageKind::Filter(handler),
            });
        }

        return Err(anyhow!("Unrecognized filter argument: {:?}", tokens));
    }

    /// Parses a flat list of merge strategy arguments into a `Stage`
    pub fn try_from_strings_merge_strategy(
        mut tokens: VecDeque<String>,
        _jinja: &JinjaEngine,
    ) -> Result<Stage> {
        let path = tokens
            .pop_front()
            .ok_or_else(|| anyhow!("Missing path for merge strategy"))?;
        let strategy = tokens
            .pop_front()
            .ok_or_else(|| anyhow!("Missing strategy name for merge strategy"))?;
        let mut args_deque = VecDeque::new();
        args_deque.push_back(strategy);
        for arg in tokens {
            args_deque.push_back(arg);
        }
        Ok(Stage::new(StageKind::MergeStrategy {
            path,
            strategy: args_deque,
        }))
    }
}
