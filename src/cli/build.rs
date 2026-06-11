use anyhow::Result;
use clap::Args;

use crate::workflow::Workflow;

/// Raw arguments for the build command.
///
/// Arguments are used to describe inputs, outputs, filters, and merge strategies.
/// Use `--input` / `-i` to start an input spec, `--output` / `-o` to start the output spec,
/// `--filter` / `-f` to apply a filter, and `--merge-strategy` / `-m` to add a merge strategy.
///
/// Example:
///   --input file /path yaml -m my_attribute.my_subattribute merge_by_key name --output stdio json
#[derive(Args)]
pub struct BuildArgs {
    /// Positional arguments describing inputs, outputs, filters, and merge strategies.
    #[arg(value_name = "TOKENS", num_args = 1.., trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

pub fn build(build_args: BuildArgs) -> Result<()> {
    let workflow = Workflow::try_from_args(build_args.args)?;
    workflow.execute()?;
    Ok(())
}
