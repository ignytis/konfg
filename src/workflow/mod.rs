pub mod io;
pub mod stage;

use std::collections::{LinkedList, VecDeque};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    jinja::JinjaEngine,
    utils::{cfg_values::cfg_values_deep_merge, hashmap::hashmap_new_from_kv_params},
    workflow::stage::Stage,
};

/// Constants for command-line arguments.
const TOKEN_INPUT_SHORT: &str = "-i";
const TOKEN_INPUT_LONG: &str = "--input";
const TOKEN_OUTPUT_SHORT: &str = "-o";
const TOKEN_OUTPUT_LONG: &str = "--output";
const TOKEN_PARAM_SHORT: &str = "-p";
const TOKEN_PARAM_LONG: &str = "--param";

/// Represents a configuration building workflow.
pub struct Workflow {
    /// List of stages to be executed sequentially.
    /// The last stage is always the output stage.
    pub stages: LinkedList<Stage>,
    /// Parameters used for Jinja2 templating.
    pub params: Vec<String>,
}

impl Workflow {
    /// Builds a workflow from a list of command-line arguments.
    pub fn try_from_args(args: Vec<String>) -> Result<Self> {
        let mut stages = LinkedList::new();
        let mut params: Vec<String> = Vec::new();

        let jinja = JinjaEngine::new();

        let mut queue: VecDeque<String> = args.into_iter().collect();
        while let Some(tok) = queue.pop_front() {
            match tok.as_str() {
                TOKEN_INPUT_SHORT | TOKEN_INPUT_LONG => {
                    let buf = parse_arg_buffer(&mut queue);
                    stages.push_back(Stage::try_from_strings(buf, &jinja, false)?);
                }
                TOKEN_OUTPUT_SHORT | TOKEN_OUTPUT_LONG => {
                    let buf = parse_arg_buffer(&mut queue);
                    stages.push_back(Stage::try_from_strings(buf, &jinja, true)?);
                }
                TOKEN_PARAM_SHORT | TOKEN_PARAM_LONG => match queue.pop_front() {
                    Some(p) => params.push(p),
                    None => {
                        return Err(anyhow!("No parameter is specified after -p or --param"));
                    }
                },
                other => {
                    return Err(anyhow!("Unexpected argument: {}", other));
                }
            }
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
            stages.push_back(Stage::try_from_strings(
                VecDeque::from(vec!["stdio".to_string(), "yaml".to_string()]),
                &jinja,
                true,
            )?);
        }

        Ok(Workflow { stages, params })
    }

    /// Executes the workflow: runs all input stages, merges their results, and runs the output stage.
    pub fn execute(&self) -> Result<()> {
        let params_map = hashmap_new_from_kv_params(&self.params)?;
        let mut jinja_ctx: Value = params_map.into();
        let mut merged: Value = Value::Object(Default::default());

        let mut iter = self.stages.iter().peekable();
        while let Some(stage) = iter.next() {
            if iter.peek().is_none() {
                // Last stage (output)
                stage.run(&merged)?;
            } else {
                // Input stage
                let value = stage.run(&jinja_ctx)?;

                cfg_values_deep_merge(&mut merged, value.clone())?;
                // Update context. Values from the previous iterations could be re-used
                // in the next iterations
                cfg_values_deep_merge(&mut jinja_ctx, merged.clone())?;
            }
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
            || next == TOKEN_PARAM_SHORT
            || next == TOKEN_PARAM_LONG
        {
            break;
        }
        buf.push_back(buf_all.pop_front().unwrap());
    }

    buf
}
