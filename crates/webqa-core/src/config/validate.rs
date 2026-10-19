//! Turning the raw configuration into a checked [`Config`].

mod check;
mod secrets;
mod storage_state;

use std::io;
use std::path::Path;

use url::Url;

use super::raw::RawConfig;
use super::{Config, ConfigError, Judge, Target};
pub use check::LEAVE_OPS;
use check::check;
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

fn target_url(url: &str) -> Result<Url, ConfigError> {
    Url::parse(url).map_err(|_| ConfigError::InvalidUrl("target.url"))
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
            api_key: raw.judge.api_key.map(|r| secret(&r.env, env)).transpose()?,
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
        interview: raw.interview,
    })
}

/// Shared by the tests of validation.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::raw::parse;

    /// A storage state in Playwright's format.
    pub const STATE: &str = r#"{"cookies":[],"origins":[]}"#;

    /// The value every variable has in [`ALL_ENV`].
    pub fn value_of(name: &str) -> String {
        format!("value-of-{name}")
    }

    /// Every environment variable is set.
    pub const ALL_ENV: Env<'static> = &|name| Some(value_of(name));
    /// Every file holds [`STATE`].
    pub const STATE_OK: Read<'static> = &|_| Ok(STATE.to_owned());

    /// Parses and validates with [`ALL_ENV`] and [`STATE_OK`].
    pub fn load(yaml: &str) -> Result<Config, ConfigError> {
        validate(parse(yaml)?, ALL_ENV, STATE_OK)
    }
}
