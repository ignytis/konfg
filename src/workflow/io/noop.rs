use anyhow::Result;

use crate::workflow::{
    io::{BaseIoHandler, OutputHandler},
    stage::{Stage, StageArgs, StageExecutionContext, StageKind},
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
    pub fn new_from_args(tokens: StageArgs, is_output: bool) -> Result<Stage> {
        if tokens.args.first().map(String::as_str) != Some(KIND) {
            return Err(anyhow::anyhow!("noop handler: not supported"));
        }
        if !is_output {
            return Err(anyhow::anyhow!("noop: only supported as output handler"));
        }
        Ok(Stage::new(StageKind::Output(Box::new(NoopHandler))))
    }
}
