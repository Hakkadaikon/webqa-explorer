//! Reading the configuration into what a run needs.

pub mod redact;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fmt;
use std::mem::take;
use std::path::{Path, PathBuf};
use std::time::Duration;

use url::Url;
use webqa_core::config::validate::{Env, Read, validate};
use webqa_core::config::{self, Config, ConfigError, Secret, check_mode, raw};
use webqa_core::event::Mode;
use webqa_core::guard::domain::InvalidOrigin;
use webqa_core::machine::Rules;
use webqa_core::redact::{InvalidRule, Redacted, Redactor};

/// What the browser starts with, before the adapter's own types.
/// No `Debug`: it holds the input values.
pub struct BrowserPlan {
    /// `target.url`, where the run starts.
    pub start: Url,
    /// `auth.storage_state`, already checked for Playwright's format.
    pub storage_state: Option<PathBuf>,
    /// `limits.page_timeout_secs`.
    pub page_timeout: Duration,
    /// `guard.forbidden.selectors`.
    pub forbidden_selectors: Vec<String>,
    /// `guard.allowed.selectors`.
    pub allowed_selectors: Vec<String>,
    /// The values typed into inputs, keyed by the input's accessible name.
    pub inputs: BTreeMap<String, Secret>,
}

/// What one run needs, read from the configuration.
pub struct Setup {
    /// The kind of run.
    pub mode: Mode,
    /// The rules of the core.
    pub rules: Rules,
    /// Hides the secrets before anything leaves the process.
    pub redactor: Redactor,
    /// `limits.error_confidence`.
    pub error_confidence: f64,
    /// `Url::origin().ascii_serialization()` of `target.url`: the key of the memory.
    pub origin: String,
    /// `target.url`, redacted, for the report.
    pub target: Redacted<String>,
    /// The keys of `inputs`, for the report.
    pub inputs: Vec<String>,
    /// What the browser starts with.
    pub browser: BrowserPlan,
    /// The judge part of the checked configuration, with its pricing.
    pub judge: config::Judge,
}

impl fmt::Debug for Setup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Setup")
            .field("mode", &self.mode)
            .field("origin", &self.origin)
            .field("rules", &self.rules)
            .finish_non_exhaustive()
    }
}

/// Why the configuration cannot start a run. No variant carries a value
/// of the configuration or the environment.
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    /// The configuration was rejected.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// An allowed origin is not an origin.
    #[error("target.allowed_origins を許可するオリジンとして読めない")]
    Origin(#[from] InvalidOrigin),
    /// A redaction rule cannot be used.
    #[error(transparent)]
    Redact(#[from] InvalidRule),
}

/// Why `.env` could not be read. Carries neither its lines nor its values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("作業ディレクトリの .env を読めない")]
pub struct DotEnvError;

/// Adds the variables of `<dir>/.env` that are not set yet. No file is
/// not an error.
///
/// Call it before starting the async runtime: it changes the environment
/// of the process.
///
/// # Errors
///
/// [`DotEnvError`] when the file exists but cannot be read or parsed.
pub fn dotenv(dir: &Path) -> Result<(), DotEnvError> {
    let path = dir.join(".env");
    // When it cannot be told, reading the file reports the problem.
    if path.try_exists().unwrap_or(true) {
        // dotenvy's error may quote a line of the file, so it is dropped.
        dotenvy::from_path(&path).map_err(|_| DotEnvError)
    } else {
        Ok(())
    }
}

/// Reads `yaml` for a run in `mode`.
///
/// # Errors
///
/// [`SetupError`] for the first problem, in the order: YAML, validation,
/// the mode, the allowed origins, the redaction rules.
pub fn load(yaml: &str, mode: Mode, env: Env<'_>, read: Read<'_>) -> Result<Setup, SetupError> {
    assemble(checked(yaml, mode, env, read)?, mode)
}

/// The configuration, validated for `mode`.
fn checked(yaml: &str, mode: Mode, env: Env<'_>, read: Read<'_>) -> Result<Config, ConfigError> {
    let config = validate(raw::parse(yaml)?, env, read)?;
    check_mode(&config, mode)?;
    Ok(config)
}

/// The rules come before the redaction rules, in the order of the errors.
/// Validation already rejects an allowed origin that does not parse, so
/// [`SetupError::Origin`] is not expected here.
fn assemble(config: Config, mode: Mode) -> Result<Setup, SetupError> {
    Rules::from_config(&config, mode)
        .map_err(SetupError::from)
        .and_then(|rules| with_rules(config, mode, rules))
}

fn with_rules(mut config: Config, mode: Mode, rules: Rules) -> Result<Setup, SetupError> {
    let redactor = Redactor::new(&redact::rules(&config))?;
    let browser = browser(&mut config);
    Ok(Setup {
        mode,
        rules,
        error_confidence: config.limits.error_confidence,
        origin: browser.start.origin().ascii_serialization(),
        target: redactor.url(&browser.start),
        inputs: browser.inputs.keys().cloned().collect(),
        redactor,
        browser,
        judge: config.judge,
    })
}

/// Takes what the browser needs out of `config`.
fn browser(config: &mut Config) -> BrowserPlan {
    BrowserPlan {
        start: config.target.url.clone(),
        storage_state: config.auth.take().and_then(|auth| auth.storage_state),
        page_timeout: Duration::from_secs(config.limits.page_timeout_secs.get()),
        forbidden_selectors: take(&mut config.guard.forbidden.selectors),
        allowed_selectors: take(&mut config.guard.allowed.selectors),
        inputs: take(&mut config.inputs),
    }
}
