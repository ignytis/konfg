use std::env;

use anyhow::Result;
use minijinja::Error;

use md5::Md5;
use sha2::{Digest, Sha256, Sha512};

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

/// Returns the MD5 hash of a string
pub fn md5(input: &str) -> Result<String, Error> {
    let mut hasher = Md5::new();
    hasher.update(input);
    let result = format!("{:x}", hasher.finalize());
    Ok(result)
}

/// Returns the SHA256 hash of a string
pub fn sha256(input: &str) -> Result<String, Error> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = format!("{:x}", hasher.finalize());
    Ok(result)
}

/// Returns the SHA512 hash of a string
pub fn sha512(input: &str) -> Result<String, Error> {
    let mut hasher = Sha512::new();
    hasher.update(input);
    let result = format!("{:x}", hasher.finalize());
    Ok(result)
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

    #[test]
    fn test_md5() {
        assert_eq!(md5("hello").unwrap(), "5d41402abc4b2a76b9719d911017c592");
    }

    #[test]
    fn test_sha256() {
        assert_eq!(
            sha256("hello").unwrap(),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_sha512() {
        assert_eq!(sha512("hello").unwrap(), "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043");
    }
}
