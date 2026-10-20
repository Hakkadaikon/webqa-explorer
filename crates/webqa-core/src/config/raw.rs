//! The configuration as read from YAML, before validation.

use std::collections::BTreeMap;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ConfigError;
use super::interview::RawInterview;

/// The whole configuration file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawConfig {
    /// The service under test.
    pub target: RawTarget,
    /// The judge model.
    pub judge: RawJudge,
    /// Thresholds and limits.
    pub limits: Limits,
    /// Guardrails.
    pub guard: Guard,
    /// Values to hide.
    #[serde(default)]
    pub redact: Vec<RawRedact>,
    /// Values to type into inputs, keyed by the input's accessible name.
    #[serde(default)]
    pub inputs: BTreeMap<String, EnvRef>,
    /// Authentication state.
    pub auth: Option<Auth>,
    /// A scripted scenario.
    pub scenario: Option<Scenario>,
    /// Operation kinds in use. All eight when omitted.
    #[serde(default = "all_ops")]
    pub ops: Vec<Op>,
    /// The interview settings.
    pub interview: Option<RawInterview>,
}

/// The service under test.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTarget {
    /// The start URL.
    pub url: String,
    /// Allowed origins. A host may start with `*.`.
    pub allowed_origins: Vec<String>,
}

/// The judge model.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawJudge {
    /// The adapter kind.
    pub kind: String,
    /// The bridge command.
    pub command: Option<Vec<String>>,
    /// The environment variable holding the API key. When omitted,
    /// authentication is left outside (a relay or the bridge command).
    #[serde(default)]
    pub api_key: Option<EnvRef>,
    /// How long to wait for an answer.
    pub timeout_secs: NonZeroU64,
    /// Prices used to compute the cost.
    pub pricing: Pricing,
}

/// A reference to an environment variable.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvRef {
    /// The variable name.
    pub env: String,
}

/// Prices per 1000 tokens.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pricing {
    /// Price of 1000 input tokens.
    pub input_per_1k: f64,
    /// Price of 1000 output tokens.
    pub output_per_1k: f64,
}

/// Thresholds and limits.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// Unchanged screens in a row that count as frozen.
    pub freeze_threshold: NonZeroU32,
    /// Steps without progress that count as stuck.
    pub stuck_threshold: NonZeroU32,
    /// The most judge calls in a run.
    pub max_decisions: NonZeroU32,
    /// The most operations in an hour.
    pub requests_per_hour: NonZeroU32,
    /// How long to wait for a page.
    pub page_timeout_secs: NonZeroU64,
    /// How long a run may last. No time limit when omitted.
    pub max_duration_secs: Option<NonZeroU64>,
    /// Confidence needed to go back.
    pub back_confidence: f64,
    /// Confidence needed to record an issue.
    pub error_confidence: f64,
}

/// Guardrails.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guard {
    /// What must not be touched.
    pub forbidden: Forbidden,
    /// What may be touched despite `forbidden`.
    pub allowed: Allowed,
    /// What to do on a paywall. `back` when omitted.
    #[serde(default = "default_leave_paywall")]
    pub leave_paywall: String,
}

/// What must not be touched.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Forbidden {
    /// Keywords.
    pub keywords: Vec<String>,
    /// CSS selectors.
    pub selectors: Vec<String>,
}

/// What may be touched despite `forbidden`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Allowed {
    /// CSS selectors.
    pub selectors: Vec<String>,
}

/// A value to hide.
#[derive(Debug, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum RawRedact {
    /// A value matched by a regular expression.
    Pattern {
        /// The name written in place of the value.
        name: String,
        /// The regular expression.
        pattern: String,
    },
    /// A value read from an environment variable.
    Env {
        /// The name written in place of the value.
        name: String,
        /// The variable name.
        env: String,
    },
}

/// Authentication state.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Auth {
    /// Path to a Playwright storage state file.
    pub storage_state: Option<PathBuf>,
}

/// A scripted scenario.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    /// Stages in order.
    pub stages: Vec<Stage>,
}

/// One stage of a scenario.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    /// The stage name.
    pub name: String,
    /// The goal given to the judge.
    pub goal: String,
    /// When the stage is done.
    pub done_when: DoneWhen,
}

/// When a stage is done.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DoneWhen {
    /// The URL path reached.
    pub url: Option<String>,
    /// An element shown.
    pub element: Option<Element>,
    /// A text shown.
    pub text: Option<String>,
}

/// An element by role and accessible name.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Element {
    /// The ARIA role.
    pub role: String,
    /// The accessible name.
    pub name: String,
}

/// An operation kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    /// Click an element.
    Click,
    /// Type into an element.
    Input,
    /// Scroll down.
    ScrollDown,
    /// Scroll up.
    ScrollUp,
    /// Go back.
    Back,
    /// Press Enter where the focus is.
    Enter,
    /// Wait.
    Wait,
    /// Declare the stage done.
    Done,
    /// Declare being stuck.
    Stuck,
}

fn all_ops() -> Vec<Op> {
    vec![
        Op::Click,
        Op::Input,
        Op::ScrollDown,
        Op::ScrollUp,
        Op::Back,
        Op::Enter,
        Op::Wait,
        Op::Done,
        Op::Stuck,
    ]
}

fn default_leave_paywall() -> String {
    "back".to_owned()
}

/// Reads the YAML into the raw types. Unknown fields are rejected.
///
/// # Errors
///
/// [`ConfigError::LiteralApiKey`] when `judge.api_key` is not
/// `{ env: <name> }`, and [`ConfigError::Yaml`] for any other problem.
/// Neither shows the YAML itself.
pub fn parse(yaml: &str) -> Result<RawConfig, ConfigError> {
    reject_literal_key(yaml)?;
    Ok(serde_saphyr::from_str_with_options(yaml, options())?)
}

fn options() -> serde_saphyr::Options {
    serde_saphyr::options! { with_snippet: false }
}

/// Only `judge.api_key`, read leniently so that a key written in place never
/// reaches a type mismatch error, which would quote it.
#[derive(Deserialize)]
struct Probe {
    judge: ProbeJudge,
}

#[derive(Deserialize)]
struct ProbeJudge {
    api_key: Value,
}

fn reject_literal_key(yaml: &str) -> Result<(), ConfigError> {
    let probe = serde_saphyr::from_str_with_options::<Probe>(yaml, options());
    if probe.is_ok_and(|p| !is_env_ref(&p.judge.api_key)) {
        Err(ConfigError::LiteralApiKey)
    } else {
        Ok(())
    }
}

/// Exactly `{ env: <string> }`. Anything else may hold the key itself, and
/// the error for an unknown field or a wrong type would quote it.
fn is_env_ref(api_key: &Value) -> bool {
    api_key
        .as_object()
        .is_some_and(|map| map.len() == 1 && map.get("env").is_some_and(Value::is_string))
}

/// Shared by the tests of the configuration.
#[cfg(test)]
pub(crate) mod tests;
