use anyhow::Result;
use clap::Args;

use crate::workflow::Workflow;

/// Raw arguments for the build command.
///
/// Positional arguments are used to describe inputs and output. Use `--in` to start an input spec
/// and `--out` to start the output spec. All arguments after `--in` until the next `--in` or `--out`
/// are considered part of that input. Example:
///   --in file /path yaml --in stdio json --out yaml
#[derive(Args)]
pub struct BuildArgs {
    /// Positional arguments describing inputs and output.
    #[arg(value_name = "TOKENS", num_args = 1.., trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

pub fn build(build_args: BuildArgs) -> Result<()> {
    let workflow = Workflow::try_from_args(build_args.args)?;
    workflow.execute()?;
    Ok(())
}
