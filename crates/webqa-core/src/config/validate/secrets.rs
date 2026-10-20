//! Reading the secrets the configuration refers to.

use std::collections::{BTreeMap, BTreeSet};

use super::super::raw::{RawConfig, RawInput, RawRedact};
use super::super::{ConfigError, Redact, RedactSource, Secret};
use super::Env;

/// The input values and the values to hide.
pub struct Secrets {
    pub inputs: BTreeMap<String, Secret>,
    /// The inputs written with `secret: false`.
    pub open_inputs: BTreeSet<String>,
    pub redact: Vec<Redact>,
}

impl Secrets {
    pub fn read(raw: &RawConfig, env: Env<'_>) -> Result<Self, ConfigError> {
        Ok(Self {
            inputs: inputs(&raw.inputs, env)?,
            open_inputs: open_inputs(&raw.inputs),
            redact: redacts(&raw.redact, env)?,
        })
    }
}

/// The value of the environment variable `name`.
pub fn secret(name: &str, env: Env<'_>) -> Result<Secret, ConfigError> {
    env(name)
        .map(Secret)
        .ok_or_else(|| ConfigError::MissingEnv(name.to_owned()))
}

fn inputs(
    inputs: &BTreeMap<String, RawInput>,
    env: Env<'_>,
) -> Result<BTreeMap<String, Secret>, ConfigError> {
    inputs
        .iter()
        .map(|(name, var)| secret(&var.env, env).map(|value| (name.clone(), value)))
        .collect()
}

fn open_inputs(inputs: &BTreeMap<String, RawInput>) -> BTreeSet<String> {
    inputs
        .iter()
        .filter(|(_, input)| !input.secret)
        .map(|(name, _)| name.clone())
        .collect()
}

fn redacts(entries: &[RawRedact], env: Env<'_>) -> Result<Vec<Redact>, ConfigError> {
    entries.iter().map(|entry| redact(entry, env)).collect()
}

fn redact(entry: &RawRedact, env: Env<'_>) -> Result<Redact, ConfigError> {
    redact_source(entry, env).map(|source| Redact {
        name: redact_name(entry).to_owned(),
        source,
    })
}

fn redact_name(entry: &RawRedact) -> &str {
    match entry {
        RawRedact::Pattern { name, .. } | RawRedact::Env { name, .. } => name,
    }
}

fn redact_source(entry: &RawRedact, env: Env<'_>) -> Result<RedactSource, ConfigError> {
    match entry {
        RawRedact::Pattern { pattern, .. } => Ok(RedactSource::Pattern(pattern.clone())),
        RawRedact::Env { env: var, .. } => secret(var, env).map(RedactSource::Env),
    }
}

#[cfg(test)]
mod tests {
    use std::mem::discriminant;

    use super::super::tests::{ALL_ENV, STATE_OK, value_of};
    use super::super::validate;
    use super::*;
    use crate::config::raw::parse;
    use crate::config::raw::tests::changed;

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
            config.judge.api_key.as_ref().map(Secret::expose),
            Some("value-of-JEV_API_KEY"),
            "key read"
        );
        let names: Vec<_> = config.redact.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["社員番号", "メールアドレス"], "redact names");
        let shown = format!("{config:?}");
        assert!(!shown.contains("value-of-"), "Debug hides secrets: {shown}");
        assert!(shown.contains("Secret(***)"), "Debug marks secrets");
    }

    #[test]
    fn omitted_api_key_leaves_auth_outside() {
        let line = "  api_key: { env: JEV_API_KEY }\n";
        let yaml = changed(line, "");
        let env = |name: &str| (name != "JEV_API_KEY").then(|| value_of(name));
        let config = validate(parse(&yaml).unwrap(), &env, STATE_OK).unwrap();
        let key = config.judge.api_key.as_ref().map(Secret::expose);
        assert_eq!(key, None, "no key read");
        let shown = format!("{config:?}");
        assert!(shown.contains("api_key: None"), "no key: {shown}");
        for form in ["~", "{}"] {
            let yaml = changed(line, &format!("  api_key: {form}\n"));
            let err = parse(&yaml).map(|_| ()).unwrap_err();
            assert_eq!(
                discriminant(&err),
                discriminant(&ConfigError::LiteralApiKey),
                "{form}: {err:?}"
            );
        }
    }
}
