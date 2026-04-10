const SCHEME_SEPARATOR: &str = "://";

#[derive(Debug, Clone)]
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

    /// Tries to parse the string as URL.
    /// The `is_input` flag indicates if given string is input or output.
    /// For input default is file, for output default is stdio
    pub fn try_or_default_from_string<S: Into<String>>(input: S, is_input: bool) -> Uri {
        let input2 = input.into();
        let input3 = input2.clone();
        match Self::try_or_none_from_string(input2) {
            Some(uri) => uri,
            None => Uri {
                scheme: if is_input { "file-yaml" } else { "stdio-yaml" }.to_string(),
                path: input3,
            },
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

        let uri = Uri::try_or_none_from_string("stdio-yaml://").unwrap();
        assert_eq!(uri.scheme, "stdio-yaml");
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
