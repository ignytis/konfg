mod cli;
mod format_handlers;
mod io;
mod jinja;
mod types;
mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use serde_json::Value;

use crate::{
    cli::Cli,
    types::endpoint::Endpoint,
    utils::{
        cfg_values::cfg_values_deep_merge,
        hashmap::hashmap_new_from_kv_params,
    },
};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let jinja = jinja::JinjaEngine::new();

    let params = hashmap_new_from_kv_params(&cli.params)?;
    let output = Endpoint::parse(&cli.output, false)?;

    let mut merged: Value = Value::Object(Default::default());
    let mut jinja_ctx: serde_json::Map<String, Value> = params.clone();

    for source_uri in &cli.sources {
        let source = Endpoint::parse(source_uri, true)?;
        let raw = source.read_raw()
            .with_context(|| format!("Failed to read '{source_uri}'"))?;
        let rendered = jinja.render(&raw, &jinja_ctx)
            .with_context(|| format!("Jinja rendering failed for '{source_uri}'"))?;
        let value = source.format().parse(&rendered)
            .with_context(|| format!("Failed to parse '{source_uri}'"))?;

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
