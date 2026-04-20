use anyhow::Result;
use minijinja::Error;
use std::env;

/// Returns an environment variable. Falls back to default if specified
pub fn env(name: &str, default: Option<&str>) -> Result<String, Error> {
    match env::var(name) {
        Ok(v) => Ok(v),
        Err(_) => match default {
            Some(v) => Ok(v.to_string()),
            None => Ok(String::new()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // We want to minimize the chance of naming collision. So random prefix here
    const ENV_VAR_NAME: &str = "UKINCCTHTNCWOHIATJRS_KONFG_TESTS_TEST_VAR";

    #[test]
    fn test_env_with_existing_variable() {
        unsafe {
            env::set_var(ENV_VAR_NAME, "value");
        }
        assert_eq!(env(ENV_VAR_NAME, None).unwrap(), "value".to_string());
        unsafe {
            env::remove_var(ENV_VAR_NAME);
        }
    }

    #[test]
    fn test_env_with_non_existing_variable_and_default() {
        assert_eq!(
            env("NON_EXISTING_VAR", Some("default")).unwrap(),
            "default".to_string()
        );
    }

    #[test]
    fn test_env_with_non_existing_variable_no_default() {
        assert_eq!(env("NON_EXISTING_VAR", None).unwrap(), "".to_string());
    }
}
