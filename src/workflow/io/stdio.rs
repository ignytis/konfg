use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::{
    handlers::format::get_handler_for_format,
    jinja::JinjaEngine,
    workflow::io::{IoHandler, Stage, TryParseResult},
};

const KIND: &str = "stdio";

/// Handles standard input/output operations.
#[derive(Clone)]
pub struct StdioHandler;

impl IoHandler for StdioHandler {
    fn read(
        &self,
        args: &HashMap<String, String>,
        jinja: &JinjaEngine,
        context: &serde_json::Value,
    ) -> Result<Value> {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        let rendered = jinja.render(&buf, context)?;

        match args.get("format") {
            Some(f) => get_handler_for_format(f)
                .ok_or_else(|| anyhow!("Format handler not found for: {}", f))?
                .parse(&rendered),
            None => Err(anyhow!(
                "Inputs/outputs without defined formats are not supported"
            )),
        }
    }

    fn write(&self, content: &str, _args: &HashMap<String, String>) -> Result<()> {
        std::io::stdout().write_all(content.as_bytes())?;
        Ok(())
    }

    fn supports(&self, kind: &str) -> bool {
        kind == KIND
    }

    fn clone_box(&self) -> Box<dyn IoHandler> {
        Box::new(self.clone())
    }

    fn try_parse_args(&self, tokens: &mut VecDeque<String>, jinja: &JinjaEngine) -> TryParseResult {
        if tokens.front().map(String::as_str) != Some(KIND) {
            return TryParseResult::NotSupported;
        }
        tokens.pop_front();
        let format = match tokens.pop_front() {
            Some(v) => v,
            None => return TryParseResult::Error(anyhow!("stdio: missing format")),
        };

        let mut args = HashMap::new();
        args.insert("format".to_string(), format);

        TryParseResult::Success(Stage::new(self.clone_box(), args, jinja.clone()))
    }
}
