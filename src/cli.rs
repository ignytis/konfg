use anyhow::{anyhow, Result};
use clap::Parser;

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
///
/// Inputs are specified as repeated `-in` flags, each consuming 2 or 3 tokens:
///   `-in stdio json`  or  `-in file /path/to/file.yaml yaml`
///
/// Output is specified once with `-out`:
///   `-out yaml`  or  `-out stdio json`  or  `-out file /path/to/out.json json`
#[derive(Parser)]
#[command(name = "konfg", about = "Configuration builder")]
pub struct Cli {
    /// Input spec tokens. Repeat for multiple inputs.
    /// Usage: `-in file /path fmt` or `-in stdio fmt`
    #[arg(
        short = 'i',
        long = "in",
        num_args = 1..=3,
        action = clap::ArgAction::Append,
        value_names = ["KIND", "PATH_OR_FORMAT", "FORMAT"]
    )]
    pub inputs: Vec<String>,

    /// Output spec tokens.
    /// Usage: `-out fmt` or `-out stdio fmt` or `-out file /path fmt`
    #[arg(
        short = 'o',
        long = "out",
        num_args = 1..=3,
        value_names = ["KIND_OR_FORMAT", "PATH", "FORMAT"]
    )]
    pub output: Vec<String>,

    /// Parameters available in Jinja context as `key=value`.
    #[arg(short = 'p', long = "param")]
    pub params: Vec<String>,
}
