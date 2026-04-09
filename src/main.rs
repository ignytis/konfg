mod cli;
mod format_handlers;
mod io;
mod jinja;
mod output;
mod types;
mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use serde_json::Value;

use crate::{
    cli::Cli,
    format_handlers::get_handler,
    output::OutputTarget,
    types::format::Format,
    utils::{
        cfg_values::cfg_values_deep_merge,
        hashmap::hashmap_new_from_kv_params,
        uri::Uri,
    },
};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let jinja = jinja::JinjaEngine::new();

    let params = hashmap_new_from_kv_params(&cli.params)?;
    let output = OutputTarget::parse(&cli.output)?;

    let mut merged: Value = Value::Object(Default::default());
    let mut jinja_ctx: serde_json::Map<String, Value> = params.clone();

    for source_uri in &cli.sources {
        let raw = read_source(source_uri)?;
        let rendered = jinja.render(&raw, &jinja_ctx)
            .with_context(|| format!("Jinja rendering failed for '{source_uri}'"))?;

        let fmt = Format::try_detect_format(source_uri)?;
        let value = get_handler(fmt)
            .parse(&rendered)
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

fn read_source(source_uri: &str) -> Result<String> {
    let uri = Uri::try_or_default_from_string(source_uri, true);
    let handler = io::get_handler(&uri.scheme)?;
    handler.read(&uri.path)
}
