pub mod filters;
pub mod io;
pub mod stage;

use std::collections::{LinkedList, VecDeque};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    jinja::JinjaEngine,
    utils::cfg_values::cfg_values_deep_merge,
    workflow::stage::{Stage, StageExecutionContext, StageKind},
};

/// Constants for command-line arguments.
const TOKEN_INPUT_SHORT: &str = "-i";
const TOKEN_INPUT_LONG: &str = "--input";
const TOKEN_OUTPUT_SHORT: &str = "-o";
const TOKEN_OUTPUT_LONG: &str = "--output";
const TOKEN_FILTER_SHORT: &str = "-f";
const TOKEN_FILTER_LONG: &str = "--filter";
const TOKEN_MERGE_STRATEGY_SHORT: &str = "-m";
const TOKEN_MERGE_STRATEGY_LONG: &str = "--merge-strategy";

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
            let stage_creator_fn = match tok.as_str() {
                TOKEN_INPUT_SHORT | TOKEN_INPUT_LONG => Stage::try_from_strings_input,
                TOKEN_OUTPUT_SHORT | TOKEN_OUTPUT_LONG => Stage::try_from_strings_output,
                TOKEN_FILTER_SHORT | TOKEN_FILTER_LONG => Stage::try_from_strings_filter,
                TOKEN_MERGE_STRATEGY_SHORT | TOKEN_MERGE_STRATEGY_LONG => {
                    Stage::try_from_strings_merge_strategy
                }
                other => return Err(anyhow!("Unexpected argument: {}", other)),
            };
            stages.push_back(stage_creator_fn(buf, &jinja)?);
        }

        // Validation: Ensure there is at least one input stage
        let has_input = stages.iter().any(|s| s.is_input());
        if !has_input {
            return Err(anyhow!("No input stages provided"));
        }

        // Validation: The first stage must be an input or merge strategy stage
        match stages.front() {
            Some(first)
                if first.is_input() || matches!(first.kind, StageKind::MergeStrategy { .. }) =>
            {
                ()
            }
            Some(_) => {
                return Err(anyhow!(
                    "The first stage must be an input or merge strategy stage"
                ));
            }
            None => return Err(anyhow!("No stages provided")),
        }

        // Validation: The last stage must be an output stage. If not, add default.
        let last_is_output = stages.back().map_or(false, |last| last.is_output());
        if !last_is_output {
            stages.push_back(Stage::new_output_default());
        }

        Ok(Workflow { stages })
    }

    /// Executes the workflow: runs all input stages, merges their results, and runs the output stage.
    pub fn execute(&self) -> Result<Value> {
        let mut context = StageExecutionContext::new();
        let mut iter = self.stages.iter().peekable();
        while let Some(stage) = iter.next() {
            let value = stage.run(&mut context)?;
            // Update the context for results of input stage only.
            // Output stage returns Null and cannot be merged into compiled config
            match stage.kind {
                StageKind::Input(_) => cfg_values_deep_merge(
                    &mut context.current_config,
                    &value,
                    &context.merge_strategies,
                )?,
                _ => {}
            };
        }

        // Return the final state. Although it had been possibly printed via output,
        // this value could be used in caller method
        Ok(context.current_config)
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
            || next == TOKEN_MERGE_STRATEGY_SHORT
            || next == TOKEN_MERGE_STRATEGY_LONG
        {
            break;
        }
        buf.push_back(buf_all.pop_front().unwrap());
    }

    buf
}

#[cfg(test)]
mod tests {
    use crate::{
        utils::cfg_values::cfg_values_deep_merge,
        workflow::{
            Workflow,
            stage::{StageExecutionContext, StageKind},
        },
    };

    use anyhow::Result;
    use serde_json::json;

    #[test]
    fn test_workflow_parse_merge_strategy() -> Result<()> {
        let args = vec![
            "-i".to_string(),
            "param".to_string(),
            "my_attribute.my_subattribute".to_string(),
            "val1".to_string(),
            "-m".to_string(),
            "my_attribute.my_subattribute".to_string(),
            "merge_by_key".to_string(),
            "name".to_string(),
            "-o".to_string(),
            "noop".to_string(),
        ];

        let workflow = Workflow::try_from_args(args)?;
        assert_eq!(workflow.stages.len(), 3);

        let mut stages_iter = workflow.stages.iter();
        let first = stages_iter.next().unwrap();
        assert!(first.is_input());

        let second = stages_iter.next().unwrap();
        if let StageKind::MergeStrategy { path, strategy } = &second.kind {
            assert_eq!(path, "my_attribute.my_subattribute");
            assert_eq!(strategy.get(0).unwrap(), "merge_by_key");
            assert_eq!(strategy.get(1).unwrap(), "name");
            assert_eq!(strategy.len(), 2);
        } else {
            panic!("Expected MergeStrategy stage");
        }

        Ok(())
    }

    #[test]
    fn test_workflow_execute_with_merge_strategy() -> Result<()> {
        let args = vec![
            "-i".to_string(),
            "param".to_string(),
            "a.b".to_string(),
            "c".to_string(),
            "-m".to_string(),
            "a".to_string(),
            "overwrite".to_string(),
            "-i".to_string(),
            "param".to_string(),
            "a.d".to_string(),
            "e".to_string(),
            "-o".to_string(),
            "noop".to_string(),
        ];

        let workflow = Workflow::try_from_args(args)?;
        let mut context = StageExecutionContext::new();

        for stage in &workflow.stages {
            let value = stage.run(&mut context)?;
            if stage.is_input() {
                cfg_values_deep_merge(
                    &mut context.current_config,
                    &value,
                    &context.merge_strategies,
                )?;
            }
        }

        // With overwrite strategy on "a", the second input (a.d = e) should completely overwrite the first (a.b = c).
        assert_eq!(context.current_config, json!({ "a": { "d": "e" } }));

        Ok(())
    }

    #[test]
    fn test_workflow_execute_with_merge_strategies_reset() -> Result<()> {
        let args = vec![
            "-i".to_string(),
            "param".to_string(),
            "a.b".to_string(),
            "c".to_string(),
            "-m".to_string(),
            "a".to_string(),
            "overwrite".to_string(),
            "-f".to_string(),
            "merge_strategies_reset".to_string(),
            "-i".to_string(),
            "param".to_string(),
            "a.d".to_string(),
            "e".to_string(),
            "-o".to_string(),
            "noop".to_string(),
        ];

        let workflow = Workflow::try_from_args(args)?;
        let mut context = StageExecutionContext::new();

        for stage in &workflow.stages {
            let value = stage.run(&mut context)?;
            if stage.is_input() {
                cfg_values_deep_merge(
                    &mut context.current_config,
                    &value,
                    &context.merge_strategies,
                )?;
            }
        }

        // With overwrite strategy reset before the second input,
        // it should merge with default simple strategy (appending/recursive merge).
        assert_eq!(
            context.current_config,
            json!({ "a": { "b": "c", "d": "e" } })
        );

        Ok(())
    }
}
