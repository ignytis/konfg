use anyhow::{Result, anyhow};

use crate::{
    workflow::io::{BaseIoHandler, file_common::FilePreprocessor},
    workflow::stage::{Stage, StageArgs, StageKind},
};

pub const KIND: &str = "file";

/// Handles file input/output operations.
#[derive(Clone)]
pub struct FileHandler {
    pub path: String,
    pub format: String,
}

impl FileHandler {
    pub fn new_from_args(tokens: StageArgs, is_output: bool) -> Result<Stage> {
        if tokens.args.first().map(String::as_str) != Some(KIND) {
            // Check if it's a path for guessing (though 'file' usually requires the keyword)
            return Err(anyhow!("file handler: not supported"));
        }

        let path = match tokens.args.get(1) {
            Some(v) => v.clone(),
            None => return Err(anyhow!("file: missing path")),
        };

        let (format, _) =
            crate::workflow::io::file_common::resolve_format_from_tokens(&path, &tokens.args[2..])?;

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
    use crate::workflow::stage::StageKind;

    use super::*;

    #[test]
    fn test_file_try_parse_args_input() {
        let args = StageArgs::new_from_args(vec![
            KIND.to_string(),
            "non_existent_file.yaml".to_string(),
            "yaml".to_string(),
        ]);
        let stage = FileHandler::new_from_args(args, false).unwrap();
        assert!(matches!(stage.kind, StageKind::Input(_)));
    }

    #[test]
    fn test_file_try_parse_args_output() {
        let args = StageArgs::new_from_args(vec![
            KIND.to_string(),
            "output.json".to_string(),
            "json".to_string(),
        ]);
        let stage = FileHandler::new_from_args(args, true).unwrap();
        assert!(matches!(stage.kind, StageKind::Output(_)));
    }

    #[test]
    fn test_file_supports() {
        assert_eq!(KIND, "file");
    }
}
