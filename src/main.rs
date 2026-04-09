mod cli;
mod format_handlers;
mod output;
mod types;
mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use minijinja::Environment;
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

fn render_template(content: &str, ctx: &serde_json::Map<String, Value>) -> Result<String> {
    let mut env = Environment::new();
    env.add_template("t", content)?;
    let tmpl = env.get_template("t")?;
    Ok(tmpl.render(ctx)?)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let params = hashmap_new_from_kv_params(&cli.params)?;
    let output = OutputTarget::parse(&cli.output)?;

    let mut merged: Value = Value::Object(Default::default());
    let mut jinja_ctx: serde_json::Map<String, Value> = params.clone();

    for source in &cli.sources {
        let raw = read_raw(source)?;
        let rendered = render_template(&raw, &jinja_ctx)
            .with_context(|| format!("Jinja rendering failed for '{source}'"))?;

        let fmt = Format::try_detect_format(source)?;
        let value = get_handler(fmt)
            .parse(&rendered)
            .with_context(|| format!("Failed to parse '{source}'"))?;

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

fn read_raw(source_uri: &str) -> Result<String> {
    use std::io::Read;

    if let Some(uri) = Uri::try_or_none_from_string(source_uri) {
        if uri.scheme.starts_with("stdin") {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            return Ok(buf);
        }
        return Ok(std::fs::read_to_string(uri.path)?);
    }
    Ok(std::fs::read_to_string(source_uri)?)
}
