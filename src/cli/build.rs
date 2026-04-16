use std::collections::VecDeque;

use anyhow::Result;
use clap::Args;
use serde_json::Value;

use crate::{
    handlers::io::parse_tokens,
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
    #[arg(value_name = "TOKENS", num_args = 1.., trailing_var_arg = true, allow_hyphen_values = true)]
    pub tokens: Vec<String>,
}

pub fn build(args: BuildArgs) -> Result<()> {
    // Simplified token parsing: pop flags (-i/--input, -o/--output, -p/--param) and collect following
    // tokens into argument lists until the next flag or end of input.
    let mut queue: VecDeque<String> = args.tokens.into_iter().collect();

    let mut inputs: Vec<VecDeque<String>> = Vec::new();
    let mut output: Option<VecDeque<String>> = None;
    let mut params: Vec<String> = Vec::new(); // each param is a 'key=value' string

    while let Some(tok) = queue.pop_front() {
        match tok.as_str() {
            "--in" | "-i" => {
                let mut buf: VecDeque<String> = VecDeque::new();
                while let Some(next) = queue.front() {
                    if next == "--in"
                        || next == "-i"
                        || next == "--out"
                        || next == "-o"
                        || next == "--param"
                        || next == "-p"
                    {
                        break;
                    }
                    buf.push_back(queue.pop_front().unwrap());
                }
                inputs.push(buf);
            }
            "--out" | "-o" => {
                if output.is_some() {
                    return Err(anyhow::anyhow!("Output is provided multiple times"));
                }
                let mut buf: VecDeque<String> = VecDeque::new();
                while let Some(next) = queue.front() {
                    if next == "--in"
                        || next == "-i"
                        || next == "--out"
                        || next == "-o"
                        || next == "--param"
                        || next == "-p"
                    {
                        break;
                    }
                    buf.push_back(queue.pop_front().unwrap());
                }
                output = Some(buf);
            }
            "--param" | "-p" => match queue.pop_front() {
                Some(p) => params.push(p),
                None => {
                    return Err(anyhow::anyhow!(
                        "No parameter is specified after -p or --param"
                    ));
                }
            },
            other => {
                return Err(anyhow::anyhow!("Unexpected token: {}", other));
            }
        }
    }

    if inputs.is_empty() {
        return Err(anyhow::anyhow!("No input is provided"));
    }

    let inputs: Vec<Endpoint> = inputs
        .into_iter()
        .map(|tokens| parse_tokens(tokens))
        .collect::<Result<Vec<Endpoint>, _>>()?;

    let output: Endpoint = match output {
        Some(tokens) if !tokens.is_empty() => parse_tokens(tokens)?,
        _ => parse_tokens(VecDeque::from(vec![
            "stdio".to_string(),
            "yaml".to_string(),
        ]))?,
    };

    let params = hashmap_new_from_kv_params(&params)?;

    let jinja = JinjaEngine::new();

    let mut merged: Value = Value::Object(Default::default());
    let jinja_ctx: serde_json::Map<String, Value> = params.clone();

    for input in &inputs {
        let raw = input.read()?;
        let rendered = jinja.render(&raw, &jinja_ctx)?;
        let value = input.parse(rendered.as_str())?;

        cfg_values_deep_merge(&mut merged, value.clone())?;
    }

    output.write(&merged)?;
    Ok(())
}
