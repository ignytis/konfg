pub mod build;

use clap::{Parser, Subcommand};

use crate::cli::build::BuildArgs;

/// CLI arguments.
#[derive(Parser)]
#[command(name = "konfg", about = "Configuration builder")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build configuration
    Build(BuildArgs),
}
