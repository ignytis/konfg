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

#[derive(Clone, Default, PartialEq)]
enum ParsingMode {
    #[default]
    Begin,
    Input,
    Output,
    Param,
}

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
    // Parse positional tokens into input tokens and output tokens using a queue-style state machine.
    // Expected form: --in <input-tokens> [--in <input-tokens> ...] --out <output-tokens>
    let mut queue: VecDeque<String> = args.tokens.into_iter().collect();
    let mut buffer: Vec<String> = Vec::new();
    let mut mode: ParsingMode = ParsingMode::default();

    let mut inputs: Vec<Vec<String>> = Vec::new();
    let mut output: Vec<String> = Vec::new();
    let mut params: Vec<String> = Vec::new(); // each param is a 'key=value' string

    while let Some(tok) = queue.pop_front() {
        let new_mode = match tok.as_str() {
            // FIXME: buffer is not flushed here because -i .. -i ... does not change the mode
            "--in" | "-i" => ParsingMode::Input,
            "--out" | "-o" => ParsingMode::Output,
            "--param" | "-p" => ParsingMode::Param,
            _ => mode.clone(),
        };

        if tok.is_empty() || mode != new_mode {
            // End of args or mode changed: finalize the current mode
            match mode {
                ParsingMode::Begin => {}
                ParsingMode::Input => {
                    inputs.push(buffer.clone());
                }
                ParsingMode::Output => {
                    if !output.is_empty() {
                        return Err(anyhow::anyhow!("Output is provided multiple times"));
                    }
                    output = buffer.clone();
                }
                ParsingMode::Param => {
                    match buffer.len() {
                        1 => params.push(buffer.remove(0)),
                        0 => return Err(anyhow::anyhow!(
                                "No parameter is specified after -p or --param"
                            )),
                        num_params => return Err(anyhow::anyhow!(
                                "{} parameters specified after -p or --param. Expected exactly one parameter.",
                                num_params
                            )),
                    };
                }
            };
            buffer.clear();

            mode = new_mode.clone();
        } else if mode == new_mode {
            // same mode: add items to buffer
            buffer.push(tok);
        }
    }

    if inputs.is_empty() {
        return Err(anyhow::anyhow!("No input is provided"));
    }

    let inputs: Vec<Endpoint> = inputs
        .iter()
        .map(|tokens| parse_tokens(tokens))
        .collect::<Result<Vec<Endpoint>, _>>()?;

    let output: Endpoint = if output.is_empty() {
        // No output provided - set YAML + stdout as default
        parse_tokens(&vec!["stdio".to_string(), "yaml".to_string()])?
    } else {
        parse_tokens(&output)?
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
