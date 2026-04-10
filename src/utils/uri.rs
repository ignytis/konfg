const SCHEME_SEPARATOR: &str = "://";

/// A URI structure representing a scheme and a path.
pub struct Uri {
    pub scheme: String,
    pub path: String,
}

impl Uri {
    /// Tries to parse a URI from an input string. Returns None on failure.
    pub fn try_or_none_from_string<S: Into<String>>(input: S) -> Option<Uri> {
        let input = input.into();
        if let Some(colon_pos) = input.find(SCHEME_SEPARATOR) {
            let scheme = &input[..colon_pos];
            let path = &input[colon_pos + SCHEME_SEPARATOR.len()..];

            Some(Uri {
                scheme: scheme.to_string(),
                path: path.to_string(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_parse_valid() {
        let uri = Uri::try_or_none_from_string("file://path/to/file").unwrap();
        assert_eq!(uri.scheme, "file");
        assert_eq!(uri.path, "path/to/file");

        let uri = Uri::try_or_none_from_string("stdin-yaml://").unwrap();
        assert_eq!(uri.scheme, "stdin-yaml");
        assert_eq!(uri.path, "");
    }

    #[test]
    fn test_uri_parse_invalid() {
        assert!(Uri::try_or_none_from_string("no-scheme-separator").is_none());
        assert!(Uri::try_or_none_from_string("file:/missing-slash").is_none());
        assert!(Uri::try_or_none_from_string("").is_none());
    }

    #[test]
    fn test_uri_parse_multi_colon() {
        let uri = Uri::try_or_none_from_string("scheme://path://to//file").unwrap();
        assert_eq!(uri.scheme, "scheme");
        assert_eq!(uri.path, "path://to//file");
    }
}
