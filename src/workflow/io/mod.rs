pub mod env;
pub mod file;
pub mod file_common;
pub mod param;
pub mod stdio;
pub mod tplfile;

use std::{
    collections::{HashMap, VecDeque},
    sync::LazyLock,
};

use anyhow::Result;
use serde_json::Value;

use crate::{
    jinja::JinjaEngine,
    workflow::stage::{Stage, StageExecutionContext},
};

pub const REGISTERED_HANDLERS: LazyLock<Vec<Box<dyn BaseIoHandler>>> = LazyLock::new(|| {
    vec![
        Box::new(stdio::StdioHandler),
        Box::new(file::FileHandler),
        Box::new(tplfile::TplFileHandler),
        Box::new(env::EnvHandler),
        Box::new(param::ParamHandler),
    ]
});

/// Result of attempting to parse tokens by a handler.
pub enum TryParseResult {
    /// Arguments were successfully parsed into Stage
    Success(Stage),
    /// Parser does not support the provided input
    NotSupported,
    /// Parser supports the provided input, but an error occurred
    Error(anyhow::Error),
}

/// Base trait for handling input/output operations.
pub trait BaseIoHandler: Send + Sync {
    /// Checks if this handler supports the given kind, e.g. "file" or "stdio".
    fn supports(&self, kind: &str) -> bool;

    /// Clones the handler into a boxed trait object.
    fn clone_box(&self) -> Box<dyn BaseIoHandler>;

    /// Attempts to pop args and construct a `Stage`.
    /// Returns `TryParseResult::NotSupported` if the first token is not supported by this handler.
    fn try_parse_args(
        &self,
        args: &mut VecDeque<String>,
        jinja: &JinjaEngine,
        is_output: bool,
    ) -> TryParseResult;
}

/// Trait for handling input operations.
pub trait InputHandler: BaseIoHandler {
    /// Reads content from the source, rendering it as Jinja template if supported.
    fn read(
        &self,
        args: &HashMap<String, String>,
        jinja: &JinjaEngine,
        context: &StageExecutionContext,
    ) -> Result<Value>;
}

/// Trait for handling output operations.
pub trait OutputHandler: BaseIoHandler {
    /// Writes serialized content to the destination.
    fn write(
        &self,
        content: &str,
        args: &HashMap<String, String>,
        context: &StageExecutionContext,
    ) -> Result<()>;
}

impl Clone for Box<dyn BaseIoHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
