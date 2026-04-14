mod cli;
mod handlers;
mod jinja;
mod types;
mod utils;

use anyhow::Result;
use clap::Parser;

use crate::{
    cli::{Cli, Commands, build::build},
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build(args) => build(args),
    }
}

