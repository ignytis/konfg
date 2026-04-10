use anyhow::Result;

/// Supported configuration formats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Yaml,
    Json,
    Toml,
    Properties,
    Dotenv,
}

impl Format {
    /// Determines the format based on a file extension.
    pub fn from_file_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "yaml" | "yml" => Some(Self::Yaml),
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            "properties" => Some(Self::Properties),
            "env" => Some(Self::Dotenv),
            _ => None,
        }
    }

    /// Determines the format based on a URI scheme (e.g., "file-yaml", "stdin-json").
    pub fn from_scheme<S: Into<String>>(scheme: S) -> Option<Self> {
        let scheme = scheme.into();
        if scheme.ends_with("-yaml") {
            Some(Self::Yaml)
        } else if scheme.ends_with("-json") {
            Some(Self::Json)
        } else if scheme.ends_with("-toml") {
            Some(Self::Toml)
        } else if scheme.ends_with("-properties") {
            Some(Self::Properties)
        } else if scheme.ends_with("-dotenv") {
            Some(Self::Dotenv)
        } else {
            None
        }
    }

    /// Detects the configuration format from a file path's extension.
    pub fn try_detect_format_from_path<S: Into<String>>(path: S) -> Result<Format> {
        let path = path.into();
        let ext = std::path::Path::new(path.as_str())
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| {
                anyhow::anyhow!("Cannot determine format for '{path}': no file extension")
            })?;
        Format::from_file_extension(ext).ok_or_else(|| {
            anyhow::anyhow!("Cannot determine format for '{path}': unknown extension '{ext}'")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_file_extension() {
        assert_eq!(Format::from_file_extension("yaml"), Some(Format::Yaml));
        assert_eq!(Format::from_file_extension("yml"), Some(Format::Yaml));
        assert_eq!(Format::from_file_extension("JSON"), Some(Format::Json));
        assert_eq!(Format::from_file_extension("toml"), Some(Format::Toml));
        assert_eq!(
            Format::from_file_extension("properties"),
            Some(Format::Properties)
        );
        assert_eq!(Format::from_file_extension("env"), Some(Format::Dotenv));
        assert_eq!(Format::from_file_extension("txt"), None);
    }

    #[test]
    fn test_from_scheme() {
        assert_eq!(Format::from_scheme("file-yaml"), Some(Format::Yaml));
        assert_eq!(Format::from_scheme("stdin-json"), Some(Format::Json));
        assert_eq!(Format::from_scheme("custom-toml"), Some(Format::Toml));
        assert_eq!(
            Format::from_scheme("http-properties"),
            Some(Format::Properties)
        );
        assert_eq!(Format::from_scheme("s3-dotenv"), Some(Format::Dotenv));
        assert_eq!(Format::from_scheme("invalid"), None);
    }

    #[test]
    fn test_try_detect_format_from_path() {
        assert!(Format::try_detect_format_from_path("config.yaml").is_ok());
        assert!(Format::try_detect_format_from_path("data.json").is_ok());
        assert!(Format::try_detect_format_from_path("no_ext").is_err());
        assert!(Format::try_detect_format_from_path("config.unknown").is_err());
    }
}
