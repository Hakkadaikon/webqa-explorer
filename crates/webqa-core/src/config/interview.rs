//! The settings of an interview run, and the documents they point to.

pub(super) mod document;
mod error;
mod raw;
mod settings;

use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;

pub use error::InterviewError;
pub use raw::{RawInterview, RawReset};

use super::persona::{self, Persona};
use super::purpose::{self, Purpose};
use super::validate::Read;
use super::{Config, ConfigError};
use settings::Settings;

/// The checked settings of an interview run.
#[derive(Debug, Clone, PartialEq)]
pub struct Interview {
    /// The person the judge plays.
    pub persona: Persona,
    /// The purpose. Inferred when no purpose document is given.
    pub purpose: Purpose,
    /// How many trials to run. At most 100.
    pub trials: NonZeroU32,
    /// How hesitant a step must be to count. In `(0, 1]`.
    pub hesitation_threshold: f64,
    /// Confidence below which a step counts as hesitant. In `(0, 1]`.
    pub confidence_threshold: f64,
    /// Steps without progress that count as stuck, from `interview` or else
    /// `limits`.
    pub stuck_threshold: NonZeroU32,
    /// The most operations in an hour, from `interview` or else `limits`.
    /// At least 2.
    pub requests_per_hour: NonZeroU32,
    /// The most questions about naturalness.
    pub max_natural_questions: NonZeroU32,
    /// The command that resets the service before each trial.
    pub reset: Option<Reset>,
}

/// The command that resets the service before each trial.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reset {
    /// The program: the first item of the command. Never blank.
    pub program: String,
    /// The arguments as written, empty ones included.
    pub args: Vec<String>,
    /// How long to wait for the command.
    pub timeout_secs: NonZeroU64,
}

/// Checks the interview settings and reads the documents they point to.
///
/// # Errors
///
/// The first problem found, as a [`ConfigError`] without any file contents,
/// paths or command arguments.
pub fn load(config: &Config, read: Read<'_>) -> Result<Interview, ConfigError> {
    let raw = config.interview.as_ref().ok_or(InterviewError::Missing)?;
    settings::check(raw, &config.limits)?.with_documents(raw, read)
}

impl Settings {
    /// The interview with these settings and the documents read.
    fn with_documents(self, raw: &RawInterview, read: Read<'_>) -> Result<Interview, ConfigError> {
        let persona = read_persona(raw, read)?;
        let purpose = read_purpose(raw, read)?;
        Ok(Interview {
            persona,
            purpose,
            trials: self.trials,
            hesitation_threshold: self.hesitation_threshold,
            confidence_threshold: self.confidence_threshold,
            stuck_threshold: self.stuck_threshold,
            requests_per_hour: self.requests_per_hour,
            max_natural_questions: self.max_natural_questions,
            reset: self.reset,
        })
    }
}

fn read_persona(raw: &RawInterview, read: Read<'_>) -> Result<Persona, ConfigError> {
    let text = read(&raw.persona).map_err(|_| InterviewError::Unreadable("interview.persona"))?;
    persona::parse(&text)
}

/// The purpose document if given, or else an inferred purpose.
fn read_purpose(raw: &RawInterview, read: Read<'_>) -> Result<Purpose, ConfigError> {
    raw.purpose
        .as_ref()
        .map_or_else(|| Ok(Purpose::inferred()), |path| purpose_file(path, read))
}

fn purpose_file(path: &Path, read: Read<'_>) -> Result<Purpose, ConfigError> {
    let text = read(path).map_err(|_| InterviewError::Unreadable("interview.purpose"))?;
    purpose::parse(&text)
}

#[cfg(test)]
pub(crate) mod tests;
