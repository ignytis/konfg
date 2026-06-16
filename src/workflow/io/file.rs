use std::collections::VecDeque;

use anyhow::{Result, anyhow};

use crate::{
    workflow::io::{BaseIoHandler, file_common::FilePreprocessor},
    workflow::stage::{Stage, StageKind},
};

pub const KIND: &str = "file";

/// Handles file input/output operations.
#[derive(Clone)]
pub struct FileHandler {
    pub path: String,
    pub format: String,
}

impl FileHandler {
    pub fn new_from_args(
        mut tokens: VecDeque<String>,
        is_output: bool,
    ) -> Result<Stage> {
        if tokens.front().map(String::as_str) != Some(KIND) {
            // Check if it's a path for guessing (though 'file' usually requires the keyword)
            return Err(anyhow!("file handler: not supported"));
        }

        tokens.pop_front();

        let path = match tokens.pop_front() {
            Some(v) => v,
            None => return Err(anyhow!("file: missing path")),
        };

        let format =
            crate::workflow::io::file_common::resolve_format_from_tokens(&path, &mut tokens)?;

        let handler = FileHandler { path, format };

        let kind = if is_output {
            StageKind::Output(Box::new(handler))
        } else {
            StageKind::Input(Box::new(handler))
        };

        Ok(Stage::new(kind))
    }
}

impl BaseIoHandler for FileHandler {}

impl FilePreprocessor for FileHandler {
    fn get_path(&self) -> &str {
        &self.path
    }

    fn get_format(&self) -> &str {
        &self.format
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::workflow::stage::StageKind;

    use super::*;

    #[test]
    fn test_file_try_parse_args_input() {
        let tokens = VecDeque::from(vec![
            KIND.to_string(),
            "non_existent_file.yaml".to_string(),
            "yaml".to_string(),
        ]);
        let stage = FileHandler::new_from_args(tokens, false).unwrap();
        assert!(matches!(stage.kind, StageKind::Input(_)));
    }

    #[test]
    fn test_file_try_parse_args_output() {
        let tokens = VecDeque::from(vec![
            KIND.to_string(),
            "output.json".to_string(),
            "json".to_string(),
        ]);
        let stage = FileHandler::new_from_args(tokens, true).unwrap();
        assert!(matches!(stage.kind, StageKind::Output(_)));
    }

    #[test]
    fn test_file_supports() {
        assert_eq!(KIND, "file");
    }
}
