//! Reading the secrets the configuration refers to.

use std::collections::BTreeMap;

use super::super::raw::{EnvRef, RawConfig, RawRedact};
use super::super::{ConfigError, Redact, RedactSource, Secret};
use super::Env;

/// The input values and the values to hide.
pub struct Secrets {
    pub inputs: BTreeMap<String, Secret>,
    pub redact: Vec<Redact>,
}

impl Secrets {
    pub fn read(raw: &RawConfig, env: Env<'_>) -> Result<Self, ConfigError> {
        Ok(Self {
            inputs: inputs(&raw.inputs, env)?,
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
    inputs: &BTreeMap<String, EnvRef>,
    env: Env<'_>,
) -> Result<BTreeMap<String, Secret>, ConfigError> {
    inputs
        .iter()
        .map(|(name, var)| secret(&var.env, env).map(|value| (name.clone(), value)))
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
