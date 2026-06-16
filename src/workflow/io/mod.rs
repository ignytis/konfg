pub mod cmd;
pub mod env;
pub mod file;
pub mod file_common;
pub mod noop;
pub mod param;
pub mod stdio;
pub mod tplfile;

use std::sync::LazyLock;

use anyhow::Result;
use serde_json::Value;

use crate::workflow::stage::{Stage, StageArgs, StageExecutionContext};

pub type IoHandlerCreatorFn = fn(StageArgs, bool) -> Result<Stage>;

pub const REGISTERED_HANDLERS: LazyLock<Vec<(&'static str, IoHandlerCreatorFn)>> =
    LazyLock::new(|| {
        vec![
            (stdio::KIND, stdio::StdioHandler::new_from_args),
            (cmd::KIND, cmd::CmdHandler::new_from_args),
            (file::KIND, file::FileHandler::new_from_args),
            (tplfile::KIND, tplfile::TplFileHandler::new_from_args),
            (env::KIND, env::EnvHandler::new_from_args),
            (param::KIND, param::ParamHandler::new_from_args),
            (noop::KIND, noop::NoopHandler::new_from_args),
        ]
    });

/// Base trait for handling input/output operations.
pub trait BaseIoHandler: Send + Sync {}

/// Trait for handling input operations.
pub trait InputHandler: BaseIoHandler {
    /// Reads content from the source, rendering it as Jinja template if supported.
    fn read(&self, context: &StageExecutionContext) -> Result<Value>;
}

/// Trait for handling output operations.
pub trait OutputHandler: BaseIoHandler {
    /// Writes serialized content to the destination.
    fn write(&self, content: &str, context: &StageExecutionContext) -> Result<()>;

    /// Returns the format name for this output handler.
    fn get_format(&self) -> Option<String>;
}
