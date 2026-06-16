use std::collections::VecDeque;

use anyhow::{Result, anyhow};

use crate::{
    jinja::JinjaEngine,
    workflow::io::{BaseIoHandler, file_common::FilePreprocessor},
    workflow::stage::{Stage, StageArgs, StageExecutionContext, StageKind},
};

pub const KIND: &str = "tplfile";

/// Handles template file input/output operations (renders Jinja templates).
#[derive(Clone)]
pub struct TplFileHandler {
    pub path: String,
    pub format: String,
}

impl TplFileHandler {
    pub fn new_from_args(tokens: StageArgs, is_output: bool) -> Result<Stage> {
        let mut args = VecDeque::from(tokens.args);
        let is_first_token_kind_keyword = match args.front().map(String::as_str) {
            Some(k) if k == KIND => true,
            Some(maybe_path) => {
                if !std::path::Path::new(maybe_path).exists() {
                    return Err(anyhow!("tplfile handler: not supported"));
                }
                false
            }
            _ => return Err(anyhow!("tplfile handler: not supported")),
        };

        if is_first_token_kind_keyword {
            args.pop_front();
        }

        let path = match args.pop_front() {
            Some(v) => v,
            None => return Err(anyhow!("tplfile: missing path")),
        };

        let format =
            crate::workflow::io::file_common::resolve_format_from_tokens(&path, &mut args)?;

        let handler = TplFileHandler { path, format };

        let kind = if is_output {
            StageKind::Output(Box::new(handler))
        } else {
            StageKind::Input(Box::new(handler))
        };

        Ok(Stage::new(kind))
    }
}

impl FilePreprocessor for TplFileHandler {
    fn preprocess(&self, raw: &str, path: &str, context: &StageExecutionContext) -> Result<String> {
        JinjaEngine::get_singleton().render(raw, path, &context.current_config)
    }

    fn get_path(&self) -> &str {
        &self.path
    }

    fn get_format(&self) -> &str {
        &self.format
    }
}

impl BaseIoHandler for TplFileHandler {}

#[cfg(test)]
mod tests {
    use crate::workflow::stage::StageKind;

    use super::*;

    #[test]
    fn test_tplfile_try_parse_args_input() {
        let args = StageArgs::new_from_args(vec![
            "tplfile".to_string(),
            "non_existent_file.yaml".to_string(),
            "yaml".to_string(),
        ]);
        let stage = TplFileHandler::new_from_args(args, false).unwrap();
        assert!(matches!(stage.kind, StageKind::Input(_)));
    }

    #[test]
    fn test_tplfile_try_parse_args_output() {
        let args = StageArgs::new_from_args(vec![
            "tplfile".to_string(),
            "output.json".to_string(),
            "json".to_string(),
        ]);
        let stage = TplFileHandler::new_from_args(args, true).unwrap();
        assert!(matches!(stage.kind, StageKind::Output(_)));
    }

    #[test]
    fn test_tplfile_supports() {
        assert_eq!(KIND, "tplfile");
    }
}
