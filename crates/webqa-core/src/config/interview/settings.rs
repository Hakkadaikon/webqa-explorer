//! Checking the values of the `interview` section.

use std::num::NonZeroU32;

use super::super::ConfigError;
use super::super::raw::Limits;
use super::document::is_blank;
use super::{InterviewError, RawInterview, RawReset, Reset};

/// The most trials in a run.
const MAX_TRIALS: u32 = 100;

/// The checked values of the `interview` section, with the limits it leaves
/// to `limits` resolved.
pub struct Settings {
    /// How many trials to run.
    pub trials: NonZeroU32,
    /// How hesitant a step must be to count.
    pub hesitation_threshold: f64,
    /// Confidence below which a step counts as hesitant.
    pub confidence_threshold: f64,
    /// Steps without progress that count as stuck.
    pub stuck_threshold: NonZeroU32,
    /// The most operations in an hour.
    pub requests_per_hour: NonZeroU32,
    /// The most questions about naturalness.
    pub max_natural_questions: NonZeroU32,
    /// The reset command.
    pub reset: Option<Reset>,
}

/// Checks the values in the order written in the documentation.
pub fn check(raw: &RawInterview, limits: &Limits) -> Result<Settings, ConfigError> {
    ranges(raw)?;
    resolve(raw, limits).map_err(ConfigError::from)
}

fn ranges(raw: &RawInterview) -> Result<(), ConfigError> {
    trials(raw.trials)?;
    thresholds(raw)
}

const fn trials(trials: NonZeroU32) -> Result<(), InterviewError> {
    if trials.get() > MAX_TRIALS {
        Err(InterviewError::Trials)
    } else {
        Ok(())
    }
}

fn thresholds(raw: &RawInterview) -> Result<(), ConfigError> {
    confidence("interview.hesitation_threshold", raw.hesitation_threshold)?;
    confidence("interview.confidence_threshold", raw.confidence_threshold)
}

/// A finite value in `(0, 1]`, as for the thresholds of `limits`.
fn confidence(field: &'static str, value: f64) -> Result<(), ConfigError> {
    if value > 0.0 && value <= 1.0 {
        Ok(())
    } else {
        Err(ConfigError::Confidence(field))
    }
}

fn resolve(raw: &RawInterview, limits: &Limits) -> Result<Settings, InterviewError> {
    let requests_per_hour = requests(raw.requests_per_hour, limits.requests_per_hour)?;
    let reset = raw.reset.as_ref().map(reset).transpose()?;
    Ok(Settings {
        trials: raw.trials,
        hesitation_threshold: raw.hesitation_threshold,
        confidence_threshold: raw.confidence_threshold,
        stuck_threshold: raw.stuck_threshold.unwrap_or(limits.stuck_threshold),
        requests_per_hour,
        max_natural_questions: raw.max_natural_questions,
        reset,
    })
}

/// The requests per hour in use, at least 2.
fn requests(
    interview: Option<NonZeroU32>,
    limits: NonZeroU32,
) -> Result<NonZeroU32, InterviewError> {
    let (field, value) = interview.map_or(("limits.requests_per_hour", limits), |value| {
        ("interview.requests_per_hour", value)
    });
    if value.get() < 2 {
        Err(InterviewError::TooFewRequests(field))
    } else {
        Ok(value)
    }
}

/// The program and its arguments. The program is never blank.
fn reset(raw: &RawReset) -> Result<Reset, InterviewError> {
    let (program, args) = raw
        .command
        .split_first()
        .filter(|(program, _)| !is_blank(program))
        .ok_or(InterviewError::MissingField("interview.reset.command"))?;
    Ok(Reset {
        program: program.clone(),
        args: args.to_vec(),
        timeout_secs: raw.timeout_secs,
    })
}
