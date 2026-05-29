pub mod filters;
pub mod io;
pub mod stage;

use std::collections::{LinkedList, VecDeque};

use anyhow::{Result, anyhow};

use crate::{
    jinja::JinjaEngine,
    utils::cfg_values::cfg_values_deep_merge,
    workflow::stage::{Stage, StageExecutionContext, StageKind, StageParserFn},
};

/// Constants for command-line arguments.
const TOKEN_INPUT_SHORT: &str = "-i";
const TOKEN_INPUT_LONG: &str = "--input";
const TOKEN_OUTPUT_SHORT: &str = "-o";
const TOKEN_OUTPUT_LONG: &str = "--output";
const TOKEN_FILTER_SHORT: &str = "-f";
const TOKEN_FILTER_LONG: &str = "--filter";

/// Represents a configuration building workflow.
pub struct Workflow {
    /// List of stages to be executed sequentially.
    /// The last stage is always the output stage.
    pub stages: LinkedList<Stage>,
}

impl Workflow {
    /// Builds a workflow from a list of command-line arguments.
    pub fn try_from_args(args: Vec<String>) -> Result<Self> {
        let jinja = JinjaEngine::new();

        let mut stages = LinkedList::new();
        let mut queue: VecDeque<String> = args.into_iter().collect();
        while let Some(tok) = queue.pop_front() {
            let buf = parse_arg_buffer(&mut queue);
            let fn_parse_args: StageParserFn = match tok.as_str() {
                TOKEN_INPUT_SHORT | TOKEN_INPUT_LONG => Stage::try_from_strings_input,
                TOKEN_OUTPUT_SHORT | TOKEN_OUTPUT_LONG => Stage::try_from_strings_output,
                TOKEN_FILTER_SHORT | TOKEN_FILTER_LONG => Stage::try_from_strings_filter,
                other => return Err(anyhow!("Unexpected argument: {}", other)),
            };
            stages.push_back(fn_parse_args(buf, &jinja)?);
        }

        // Validation: The first stage must be an input stage
        match stages.front() {
            Some(first) if first.is_input() => (),
            Some(_) => return Err(anyhow!("The first stage must be an input stage")),
            None => return Err(anyhow!("No input stages provided")),
        }

        // Validation: The last stage must be an output stage. If not, add default.
        let last_is_output = stages.back().map_or(false, |last| last.is_output());
        if !last_is_output {
            stages.push_back(Stage::new_output_default());
        }

        Ok(Workflow { stages })
    }

    /// Executes the workflow: runs all input stages, merges their results, and runs the output stage.
    pub fn execute(&self) -> Result<()> {
        let mut context = StageExecutionContext::new();
        let mut iter = self.stages.iter().peekable();
        while let Some(stage) = iter.next() {
            let value = stage.run(&mut context)?;
            // Update the context for results of input stage only.
            // Output stage returns Null and cannot be merged into compiled config
            match stage.kind {
                StageKind::Input(_) => cfg_values_deep_merge(&mut context.current_config, &value)?,
                _ => {}
            };
        }

        Ok(())
    }
}

/// Consumes the buffer of all arguments by writing arguments from there
/// into buffer for current stage until end of stage data is reached.
/// Returns a buffer with arguments for current stage.
fn parse_arg_buffer(buf_all: &mut VecDeque<String>) -> VecDeque<String> {
    let mut buf: VecDeque<String> = VecDeque::new();
    while let Some(next) = buf_all.front() {
        if next == TOKEN_INPUT_SHORT
            || next == TOKEN_INPUT_LONG
            || next == TOKEN_OUTPUT_SHORT
            || next == TOKEN_OUTPUT_LONG
            || next == TOKEN_FILTER_SHORT
            || next == TOKEN_FILTER_LONG
        {
            break;
        }
        buf.push_back(buf_all.pop_front().unwrap());
    }

    buf
}
