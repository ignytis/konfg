pub mod build;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

use crate::cli::build::BuildArgs;

/// A parsed I/O specification: either `file <path> <format>` or `stdio <format>`.
#[derive(Debug, Clone)]
pub enum IoSpec {
    File { path: String, format: String },
    Stdio { format: String },
}

/// Parses a sequence of tokens into an `IoSpec` for output, allowing a bare `<format>` shorthand.
pub fn parse_output_spec(tokens: &[String]) -> Result<IoSpec> {
    match tokens.first().map(String::as_str) {
        Some("file") if tokens.len() == 3 => Ok(IoSpec::File {
            path: tokens[1].clone(),
            format: tokens[2].clone(),
        }),
        Some("stdio") if tokens.len() == 2 => Ok(IoSpec::Stdio {
            format: tokens[1].clone(),
        }),
        Some(fmt) if tokens.len() == 1 => Ok(IoSpec::Stdio {
            format: fmt.to_string(),
        }),
        _ => Err(anyhow!(
            "Invalid -out spec {:?}. Expected: file <path> <format> | stdio <format> | <format>",
            tokens
        )),
    }
}

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
