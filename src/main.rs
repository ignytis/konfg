mod cli;
mod handlers;
mod jinja;
mod types;
mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use serde_json::Value;

use crate::{
    cli::{parse_output_spec, Cli, IoSpec},
    handlers::io::parse_specs,
    types::endpoint::Endpoint,
    utils::{cfg_values::cfg_values_deep_merge, hashmap::hashmap_new_from_kv_params},
};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let jinja = jinja::JinjaEngine::new();

    let params = hashmap_new_from_kv_params(&cli.params)?;

    let output_tokens = if cli.output.is_empty() {
        vec!["yaml".to_string()]
    } else {
        cli.output.clone()
    };
    let output_spec = parse_output_spec(&output_tokens)?;
    let output = match &output_spec {
        IoSpec::File { path, format } => Endpoint::new("file", format, path)?,
        IoSpec::Stdio { format } => Endpoint::new("stdio", format, "")?,
    };

    let mut merged: Value = Value::Object(Default::default());
    let mut jinja_ctx: serde_json::Map<String, Value> = params.clone();

    for spec in parse_specs(cli.inputs)? {
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
