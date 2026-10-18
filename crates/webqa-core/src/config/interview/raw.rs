//! The `interview` section as read from YAML, before checking.

use std::num::{NonZeroU32, NonZeroU64};
use std::path::PathBuf;

use serde::Deserialize;

/// The `interview` section of the configuration.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawInterview {
    /// Path to the persona document.
    pub persona: PathBuf,
    /// Path to the purpose document. The purpose is inferred when omitted.
    pub purpose: Option<PathBuf>,
    /// How many trials to run. 3 when omitted.
    #[serde(default = "default_trials")]
    pub trials: NonZeroU32,
    /// How hesitant a step must be to count. 0.6 when omitted.
    #[serde(default = "default_hesitation")]
    pub hesitation_threshold: f64,
    /// Confidence below which a step counts as hesitant. 0.5 when omitted.
    #[serde(default = "default_confidence")]
    pub confidence_threshold: f64,
    /// Steps without progress that count as stuck. `limits` when omitted.
    pub stuck_threshold: Option<NonZeroU32>,
    /// The most operations in an hour. `limits` when omitted.
    pub requests_per_hour: Option<NonZeroU32>,
    /// The most questions about naturalness. 50 when omitted.
    #[serde(default = "default_natural_questions")]
    pub max_natural_questions: NonZeroU32,
    /// The command that resets the service before each trial.
    pub reset: Option<RawReset>,
}

/// The command that resets the service before each trial.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawReset {
    /// The program and its arguments. Never run through a shell.
    pub command: Vec<String>,
    /// How long to wait for the command. 60 when omitted.
    #[serde(default = "default_reset_timeout")]
    pub timeout_secs: NonZeroU64,
}

const fn default_trials() -> NonZeroU32 {
    NonZeroU32::MIN.saturating_add(2)
}

const fn default_hesitation() -> f64 {
    0.6
}

const fn default_confidence() -> f64 {
    0.5
}

const fn default_natural_questions() -> NonZeroU32 {
    NonZeroU32::MIN.saturating_add(49)
}

const fn default_reset_timeout() -> NonZeroU64 {
    NonZeroU64::MIN.saturating_add(59)
}
