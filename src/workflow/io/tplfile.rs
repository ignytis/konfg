use std::collections::VecDeque;

use anyhow::Result;

use crate::{
    jinja::JinjaEngine,
    workflow::io::{
        BaseIoHandler, TryParseResult,
        file_common::{FileIoHandler, FilePreprocessor},
    },
    workflow::stage::StageExecutionContext,
};

const KIND: &str = "tplfile";

/// Handles template file input/output operations (renders Jinja templates).
#[derive(Clone)]
pub struct TplFileHandler;

impl FilePreprocessor for TplFileHandler {
    fn preprocess(
        &self,
        raw: &str,
        jinja: &JinjaEngine,
        context: &StageExecutionContext,
    ) -> Result<String> {
        jinja.render(raw, &context.current_config)
    }
}

impl BaseIoHandler for TplFileHandler {
    fn supports(&self, kind: &str) -> bool {
        kind == KIND
    }

    fn clone_box(&self) -> Box<dyn BaseIoHandler> {
        Box::new(self.clone())
    }

    fn try_parse_args(
        &self,
        tokens: &mut VecDeque<String>,
        jinja: &JinjaEngine,
        is_output: bool,
    ) -> TryParseResult {
        FileIoHandler::new(KIND, self.clone()).try_parse(tokens, jinja, is_output, true)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::{jinja::JinjaEngine, workflow::io::TryParseResult, workflow::stage::StageKind};

    use super::*;

    #[test]
    fn test_tplfile_try_parse_args_input() {
        let handler = TplFileHandler;
        let mut tokens = VecDeque::from(vec![
            "tplfile".to_string(),
            "non_existent_file.yaml".to_string(),
            "yaml".to_string(),
        ]);
        let jinja = JinjaEngine::new();

        let result = handler.try_parse_args(&mut tokens, &jinja, false);
        if let TryParseResult::Success(stage) = result {
            assert!(matches!(stage.kind, StageKind::Input(_)));
            assert_eq!(stage.args.get("path").unwrap(), "non_existent_file.yaml");
            assert_eq!(stage.args.get("format").unwrap(), "yaml");
        } else {
            panic!("Expected Success");
        }
    }

    #[test]
    fn test_tplfile_try_parse_args_output() {
        let handler = TplFileHandler;
        let mut tokens = VecDeque::from(vec![
            "tplfile".to_string(),
            "output.json".to_string(),
            "json".to_string(),
        ]);
        let jinja = JinjaEngine::new();

        let result = handler.try_parse_args(&mut tokens, &jinja, true);
        if let TryParseResult::Success(stage) = result {
            assert!(matches!(stage.kind, StageKind::Output(_)));
            assert_eq!(stage.args.get("path").unwrap(), "output.json");
            assert_eq!(stage.args.get("format").unwrap(), "json");
        } else {
            panic!("Expected Success");
        }
    }

    #[test]
    fn test_tplfile_supports() {
        let handler = TplFileHandler;
        assert!(handler.supports("tplfile"));
        assert!(!handler.supports("stdio"));
    }
}
