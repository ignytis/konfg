use std::collections::VecDeque;

use anyhow::Result;

use crate::{
    jinja::JinjaEngine,
    workflow::{
        io::{BaseIoHandler, OutputHandler},
        stage::{Stage, StageExecutionContext, StageKind},
    },
};

/// Handler identifier for the noop output handler.
pub const KIND: &str = "noop";

/// A no-operation output handler that discards the output.
/// Useful for testing: retrieve the result via `StageExecutionContext::current_config`.
pub struct NoopHandler;

impl BaseIoHandler for NoopHandler {}

impl OutputHandler for NoopHandler {
    fn write(&self, _content: &str, _context: &StageExecutionContext) -> Result<()> {
        Ok(())
    }

    fn get_format(&self) -> Option<String> {
        None
    }
}

impl NoopHandler {
    pub fn new_from_args(
        mut tokens: VecDeque<String>,
        _jinja: &JinjaEngine,
        is_output: bool,
    ) -> Result<Stage> {
        if tokens.pop_front().map(|t| t != KIND).unwrap_or(true) {
            return Err(anyhow::anyhow!("noop handler: not supported"));
        }
        if !is_output {
            return Err(anyhow::anyhow!("noop: only supported as output handler"));
        }
        Ok(Stage::new(StageKind::Output(Box::new(NoopHandler))))
    }
}
