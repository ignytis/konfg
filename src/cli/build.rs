
use anyhow::{Context, Result};
use clap::Args;
use serde_json::Value;

use crate::{
    cli::{IoSpec, parse_output_spec},
    handlers::io::parse_specs,
    jinja::JinjaEngine,
    types::endpoint::Endpoint,
    utils::{cfg_values::cfg_values_deep_merge, hashmap::hashmap_new_from_kv_params},
};

/// Arguments for the build command.
///
/// Inputs are specified as repeated `-in` flags, each consuming 2 or 3 tokens:
///   `-in stdio json`  or  `-in file /path/to/file.yaml yaml`
///
/// Output is specified once with `-out`:
///   `-out yaml`  or  `-out stdio json`  or  `-out file /path/to/out.json json`
#[derive(Args)]
pub struct BuildArgs {
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

pub fn build(args: BuildArgs) -> Result<()> {
    let jinja = JinjaEngine::new();

    let params = hashmap_new_from_kv_params(&args.params)?;

    let output_tokens = if args.output.is_empty() {
        vec!["yaml".to_string()]
    } else {
        args.output.clone()
    };
    let output_spec = parse_output_spec(&output_tokens)?;
    let output = match &output_spec {
        IoSpec::File { path, format } => Endpoint::new("file", format, path)?,
        IoSpec::Stdio { format } => Endpoint::new("stdio", format, "")?,
    };

    let mut merged: Value = Value::Object(Default::default());
    let mut jinja_ctx: serde_json::Map<String, Value> = params.clone();

    for spec in parse_specs(args.inputs)? {
        let (kind, format, path) = match &spec {
            IoSpec::File { path, format } => ("file", format.as_str(), path.as_str()),
            IoSpec::Stdio { format } => ("stdio", format.as_str(), ""),
        };
        let label = format!("{kind}:{path}");
        let source = Endpoint::new(kind, format, path)
            .with_context(|| format!("Failed to create endpoint for '{label}'"))?;

        let raw = source
            .read()
            .with_context(|| format!("Failed to read '{label}'"))?;
        let rendered = jinja
            .render(&raw, &jinja_ctx)
            .with_context(|| format!("Jinja rendering failed for '{label}'"))?;
        let value = source
            .format()
            .parse(&rendered)
            .with_context(|| format!("Failed to parse '{label}'"))?;

        cfg_values_deep_merge(&mut merged, value.clone())?;

        if let Value::Object(obj) = &value {
            for (k, v) in obj {
                if !v.is_object() && !v.is_array() {
                    jinja_ctx.insert(k.clone(), v.clone());
                }
            }
        }
    }

    output.write(&merged)
}
