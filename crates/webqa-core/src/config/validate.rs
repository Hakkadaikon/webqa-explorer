//! Turning the raw configuration into a checked [`Config`].

mod secrets;

use std::io;
use std::path::Path;

use serde_json::Value;
use url::Url;

use super::raw::{Op, RawConfig};
use super::{Config, ConfigError, Judge, Target};
use secrets::{Secrets, secret};

/// Reads an environment variable.
pub type Env<'a> = &'a dyn Fn(&str) -> Option<String>;
/// Reads a file.
pub type Read<'a> = &'a dyn Fn(&Path) -> io::Result<String>;

/// Checks the raw configuration and reads the secrets it refers to.
///
/// # Errors
///
/// The first problem found, as a [`ConfigError`] without any secret value.
pub fn validate(raw: RawConfig, env: Env<'_>, read: Read<'_>) -> Result<Config, ConfigError> {
    check(&raw, read)?;
    let secrets = Secrets::read(&raw, env)?;
    assemble(raw, secrets, env)
}

/// Checks that only reject, in the order the errors are reported.
fn check(raw: &RawConfig, read: Read<'_>) -> Result<(), ConfigError> {
    let storage_state = raw.auth.as_ref().and_then(|a| a.storage_state.as_deref());
    [
        confidence("limits.back_confidence", raw.limits.back_confidence),
        confidence("limits.error_confidence", raw.limits.error_confidence),
        non_empty_ops(&raw.ops),
        origins(&raw.target.allowed_origins),
        storage_state.map_or(Ok(()), |path| playwright_state(path, read)),
    ]
    .into_iter()
    .collect()
}

const fn confidence(field: &'static str, value: f64) -> Result<(), ConfigError> {
    if value > 0.0 && value <= 1.0 {
        Ok(())
    } else {
        Err(ConfigError::Confidence(field))
    }
}

const fn non_empty_ops(ops: &[Op]) -> Result<(), ConfigError> {
    if ops.is_empty() {
        Err(ConfigError::NoOps)
    } else {
        Ok(())
    }
}

/// Only whether each origin parses. Matching is the guardrail's job.
fn origins(origins: &[String]) -> Result<(), ConfigError> {
    origins.iter().try_for_each(|origin| {
        Url::parse(&origin.replacen("://*.", "://", 1))
            .map(drop)
            .map_err(|_| ConfigError::InvalidUrl("target.allowed_origins"))
    })
}

fn target_url(url: &str) -> Result<Url, ConfigError> {
    Url::parse(url).map_err(|_| ConfigError::InvalidUrl("target.url"))
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

fn assemble(raw: RawConfig, secrets: Secrets, env: Env<'_>) -> Result<Config, ConfigError> {
    Ok(Config {
        target: Target {
            url: target_url(&raw.target.url)?,
            allowed_origins: raw.target.allowed_origins,
        },
        judge: Judge {
            kind: raw.judge.kind,
            command: raw.judge.command,
            api_key: secret(&raw.judge.api_key.env, env)?,
            timeout_secs: raw.judge.timeout_secs,
            pricing: raw.judge.pricing,
        },
        limits: raw.limits,
        guard: raw.guard,
        redact: secrets.redact,
        inputs: secrets.inputs,
        auth: raw.auth,
        scenario: raw.scenario,
        ops: raw.ops,
    })
}

#[cfg(test)]
mod tests {
    use std::mem::discriminant;

    use super::*;
    use crate::config::raw::parse;
    use crate::config::raw::tests::{EXAMPLE, changed};

    const STATE: &str = r#"{"cookies":[],"origins":[]}"#;

    fn value_of(name: &str) -> String {
        format!("value-of-{name}")
    }

    const ALL_ENV: Env<'static> = &|name| Some(value_of(name));
    const STATE_OK: Read<'static> = &|_| Ok(STATE.to_owned());

    fn load(yaml: &str) -> Result<Config, ConfigError> {
        validate(parse(yaml)?, ALL_ENV, STATE_OK)
    }

    #[test]
    fn rejects_non_positive_and_empty() {
        assert!(load(EXAMPLE).is_ok(), "the example is valid");
        let cases = [
            ("freeze_threshold: 3", "freeze_threshold: 0"),
            ("stuck_threshold: 3", "stuck_threshold: 0"),
            ("max_decisions: 200", "max_decisions: 0"),
            ("requests_per_hour: 600", "requests_per_hour: 0"),
            ("max_decisions: 200", "max_decisions: -1"),
            ("freeze_threshold: 3", "freeze_threshold: -1"),
            ("page_timeout_secs: 10", "page_timeout_secs: 0"),
            ("timeout_secs: 30", "timeout_secs: 0"),
            ("scenario:", "ops: []\nscenario:"),
            ("back_confidence: 0.7", "back_confidence: 0"),
            ("back_confidence: 0.7", "back_confidence: 1.5"),
            ("back_confidence: 0.7", "back_confidence: .nan"),
            ("error_confidence: 0.8", "error_confidence: 0"),
            ("url: https://app.example.test", "url: not a url"),
            ("\"https://*.app.example.test\"", "\"not an origin\""),
        ];
        for (from, to) in cases {
            assert!(load(&changed(from, to)).is_err(), "{to} is rejected");
        }
        let one = changed("back_confidence: 0.7", "back_confidence: 1");
        assert!(load(&one).is_ok(), "back_confidence: 1 is accepted");
    }

    fn missing(yaml: &str, absent: &str) -> ConfigError {
        let env = |name: &str| (name != absent).then(|| value_of(name));
        validate(parse(yaml).unwrap(), &env, STATE_OK).unwrap_err()
    }

    #[test]
    fn missing_env_is_config_error() {
        let yaml = changed(
            "redact:\n",
            "redact:\n  - { name: 社員番号, env: TEST_ID }\n",
        );
        for name in ["JEV_API_KEY", "TEST_EMAIL", "TEST_ID"] {
            let err = missing(&yaml, name);
            assert!(
                matches!(&err, ConfigError::MissingEnv(v) if v == name),
                "{name}: {err:?}"
            );
        }
        reads_secrets_without_showing_them(&yaml);
    }

    fn reads_secrets_without_showing_them(yaml: &str) {
        let config = validate(parse(yaml).unwrap(), ALL_ENV, STATE_OK).unwrap();
        assert_eq!(
            config.judge.api_key.expose(),
            "value-of-JEV_API_KEY",
            "key read"
        );
        let names: Vec<_> = config.redact.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["社員番号", "メールアドレス"], "redact names");
        let shown = format!("{config:?}");
        assert!(!shown.contains("value-of-"), "Debug hides secrets: {shown}");
        assert!(shown.contains("Secret(***)"), "Debug marks secrets");
    }

    /// `None` makes reading the file fail.
    fn with_state(text: Option<&str>) -> Result<Config, ConfigError> {
        let read = |_: &Path| text.map(str::to_owned).ok_or_else(not_found);
        validate(parse(EXAMPLE).unwrap(), ALL_ENV, &read)
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
            let err = with_state(text).unwrap_err();
            assert_eq!(
                discriminant(&err),
                discriminant(&ConfigError::StorageState),
                "{err:?}"
            );
        }
        assert!(
            with_state(Some(STATE)).is_ok(),
            "Playwright's format is accepted"
        );
    }
}
