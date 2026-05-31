mod cli;
mod file_format_handlers;
mod jinja;
mod utils;
mod workflow;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Commands, build::build};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build(args) => build(args),
    }
}
