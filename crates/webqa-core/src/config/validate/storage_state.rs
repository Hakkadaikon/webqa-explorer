//! Checking the Playwright storage state file.

use std::path::Path;

use serde_json::Value;

use super::super::ConfigError;
use super::super::raw::Auth;
use super::Read;

/// Reads and checks `auth.storage_state`, if there is one.
pub fn storage_state(auth: Option<&Auth>, read: Read<'_>) -> Result<(), ConfigError> {
    auth.and_then(|a| a.storage_state.as_deref())
        .map_or(Ok(()), |path| playwright_state(path, read))
}

/// A JSON object with the `cookies` and `origins` arrays.
fn playwright_state(path: &Path, read: Read<'_>) -> Result<(), ConfigError> {
    let text = read(path).or(Err(ConfigError::StorageState))?;
    is_playwright_state(&text)
        .then_some(())
        .ok_or(ConfigError::StorageState)
}

fn is_playwright_state(text: &str) -> bool {
    serde_json::from_str::<Value>(text)
        .is_ok_and(|state| has_array(&state, "cookies") && has_array(&state, "origins"))
}

fn has_array(state: &Value, key: &str) -> bool {
    state.get(key).is_some_and(Value::is_array)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::io;
    use std::mem::discriminant;
    use std::path::PathBuf;

    use super::super::tests::{ALL_ENV, STATE};
    use super::super::validate;
    use super::*;
    use crate::config::Config;
    use crate::config::raw::parse;
    use crate::config::raw::tests::{EXAMPLE, changed};

    /// Loads `yaml` with `text` as the storage state file, `None` making
    /// reading fail. Also returns the paths read.
    fn with_state(yaml: &str, text: Option<&str>) -> (Result<Config, ConfigError>, Vec<PathBuf>) {
        let seen = RefCell::new(Vec::new());
        let read = |path: &Path| {
            seen.borrow_mut().push(path.to_owned());
            text.map(str::to_owned).ok_or_else(not_found)
        };
        let result = validate(parse(yaml).unwrap(), ALL_ENV, &read);
        (result, seen.into_inner())
    }

    fn not_found() -> io::Error {
        io::ErrorKind::NotFound.into()
    }

    #[test]
    fn malformed_storage_state_rejected() {
        let bad = [
            Some("not json"),
            Some("{}"),
            Some(r#"{"cookies":[]}"#),
            None,
        ];
        for text in bad {
            let err = with_state(EXAMPLE, text).0.unwrap_err();
            assert_eq!(
                discriminant(&err),
                discriminant(&ConfigError::StorageState),
                "{err:?}"
            );
        }
        let (result, seen) = with_state(EXAMPLE, Some(STATE));
        assert!(result.is_ok(), "Playwright's format is accepted");
        assert_eq!(
            seen,
            [PathBuf::from("./auth.json")],
            "reads the configured path"
        );
        let no_auth = changed("auth:\n  storage_state: ./auth.json\n", "");
        let (result, seen) = with_state(&no_auth, None);
        assert!(result.is_ok(), "no auth is accepted");
        assert!(seen.is_empty(), "nothing is read without auth");
    }
}
