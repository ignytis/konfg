use anyhow::{Context, Result};
use clap::Args;
use serde_json::Value;

use crate::{
    cli::{parse_output_spec, IoSpec},
    handlers::io::parse_specs,
    jinja::JinjaEngine,
    types::endpoint::Endpoint,
    utils::{cfg_values::cfg_values_deep_merge, hashmap::hashmap_new_from_kv_params},
};

/// Arguments for the build command.
///
/// Positional tokens are used to describe inputs and output. Use `--in` to start an input spec
/// and `--out` to start the output spec. All tokens after `--in` until the next `--in` or `--out`
/// are considered part of that input. Example:
///   --in file /path yaml --in stdio json --out yaml
#[derive(Args)]
pub struct BuildArgs {
    /// Positional tokens describing inputs and output.
    #[arg(value_name = "TOKENS", num_args = 1.., trailing_var_arg = true)]
    pub tokens: Vec<String>,

    /// Parameters available in Jinja context as `key=value`.
    #[arg(short = 'p', long = "param")]
    pub params: Vec<String>,
}

pub fn build(args: BuildArgs) -> Result<()> {
    let jinja = JinjaEngine::new();

    let params = hashmap_new_from_kv_params(&args.params)?;

    // Parse positional tokens into input tokens and output tokens using a queue-style state machine.
    // Expected form: --in <input-tokens> [--in <input-tokens> ...] [--out <output-tokens>]
    use std::collections::VecDeque;
    let mut queue: VecDeque<String> = args.tokens.into_iter().collect();
    let mut inputs_flat: Vec<String> = Vec::new();
    let mut output_tokens: Vec<String> = Vec::new();

    let mut mode: Option<&str> = None;
    while let Some(tok) = queue.pop_front() {
        if tok == "--in" {
            mode = Some("in");
            continue;
        } else if tok == "--out" {
            mode = Some("out");
            continue;
        }

        match mode {
            Some("in") => inputs_flat.push(tok),
            Some("out") => output_tokens.push(tok),
            None => anyhow::bail!("Unexpected token '{}' before any --in/--out", tok),
            _ => unreachable!(),
        }
    }

    if output_tokens.is_empty() {
        output_tokens = vec!["yaml".to_string()];
    }

    let output_spec = parse_output_spec(&output_tokens)?;
    let output = match &output_spec {
        IoSpec::File { path, format } => Endpoint::new("file", format, path)?,
        IoSpec::Stdio { format } => Endpoint::new("stdio", format, "")?,
    };

    let mut merged: Value = Value::Object(Default::default());
    let mut jinja_ctx: serde_json::Map<String, Value> = params.clone();

    for spec in parse_specs(inputs_flat)? {
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
