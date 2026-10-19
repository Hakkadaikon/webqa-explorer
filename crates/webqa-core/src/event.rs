//! The event log: one JSON object per line, shared by the CLI and the TUI.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::explore::MemoryFileError;
use crate::issue::{Issue, IssueClass, IssueKind};
use crate::machine::{Anomaly, Cause, ExitReason, Rejection, StuckKind};

/// One line of the event log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    /// Sequence number within the run.
    pub seq: u64,
    /// Milliseconds since the run started.
    pub elapsed_ms: u64,
    /// The step the event belongs to, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<u64>,
    /// What happened.
    #[serde(flatten)]
    pub event: Event,
}

/// The kind of run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Free exploration.
    Explore,
    /// Following a scenario.
    Scenario,
}

impl Mode {
    /// The name written to the log.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Explore => "explore",
            Self::Scenario => "scenario",
        }
    }
}

/// What a line records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    /// The first line of a run.
    RunStarted {
        /// Version of the log format.
        schema: u32,
        /// The kind of run.
        mode: Mode,
        /// Wall-clock start time, written by the CLI.
        started_at: Option<String>,
    },
    /// The memory file could not be used; the run starts from an empty memory.
    MemoryReset {
        /// Why the file could not be used.
        cause: MemoryFileError,
    },
    /// The judge was asked once.
    Asked {
        /// Tokens sent.
        input_tokens: u64,
        /// Tokens received.
        output_tokens: u64,
        /// Milliseconds the judge took.
        think_ms: u64,
    },
    /// The judge chose an operation.
    Decided {
        /// The operation.
        op: String,
        /// The target element, if the operation needs one.
        target: Option<String>,
        /// Confidence of the choice.
        p: f64,
    },
    /// The core operated the browser on its own.
    Recovered {
        /// The operation.
        op: String,
        /// Why the core operated.
        cause: Cause,
    },
    /// The browser ran the operation.
    Executed {
        /// Whether the screen changed.
        changed: bool,
        /// Number of network accesses.
        accesses: u32,
    },
    /// The guardrails stopped the judge's proposal.
    Rejected {
        /// Why it was stopped.
        why: Rejection,
    },
    /// The judge's reply could not be used.
    Anomaly {
        /// What was wrong, by name.
        cause: String,
        /// The question whose answer was wrong, if one was.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        question: Option<String>,
    },
    /// An issue was found.
    Issue {
        /// What the issue is about.
        issue: IssueKind,
        /// Whether it counts towards the exit code.
        class: IssueClass,
        /// The page string it points at, redacted.
        evidence: Option<String>,
        /// The fixed name of a semantic finding.
        label: Option<String>,
        /// The judge's confidence.
        p: Option<f64>,
        /// The file name of its screenshot.
        screenshot: String,
    },
    /// The scenario moved on to the stage `stage` (from 0) named `name`.
    StageAdvanced {
        /// The stage, from 0.
        stage: usize,
        /// Its name.
        name: String,
    },
    /// The run waits for the time window until `until_ms`.
    Waited {
        /// Milliseconds since the run started.
        until_ms: u64,
    },
    /// The last line of a run.
    RunFinished {
        /// Why the run ended.
        reason: String,
        /// How the run got stuck, when it did.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stuck: Option<StuckKind>,
    },
}

/// The fields of an anomaly line, as serde writes the anomaly.
#[derive(Debug, Default, Deserialize)]
struct AnomalyFields {
    cause: String,
    #[serde(default)]
    question: Option<String>,
}

impl Event {
    /// The last line for `reason`: its serde name, and the kind of being stuck.
    #[must_use]
    pub fn finished(reason: ExitReason) -> Self {
        let named = serde_json::to_value(reason).unwrap_or_default();
        let name = named.get("reason").and_then(Value::as_str);
        Self::RunFinished {
            reason: name.unwrap_or_default().to_owned(),
            stuck: reason.stuck_kind(),
        }
    }

    /// The line of `issue`, its screenshot saved as `screenshot`.
    #[must_use]
    pub fn found(issue: &Issue, screenshot: &str) -> Self {
        Self::Issue {
            issue: issue.kind,
            class: issue.class(),
            evidence: issue.evidence.as_ref().map(|e| e.as_inner().clone()),
            label: issue.label.clone(),
            p: issue.p,
            screenshot: screenshot.to_owned(),
        }
    }

    /// The line of an anomaly.
    #[must_use]
    pub fn anomaly(anomaly: &Anomaly) -> Self {
        let named = match anomaly {
            Anomaly::Judge(error) => serde_json::json!({ "cause": error }),
            Anomaly::Unusable(unusable) => serde_json::to_value(unusable).unwrap_or_default(),
        };
        let fields: AnomalyFields = serde_json::from_value(named).unwrap_or_default();
        Self::Anomaly {
            cause: fields.cause,
            question: fields.question,
        }
    }
}

/// Writes `line` as one line of JSON, without the trailing newline.
#[must_use]
pub fn to_json_line(line: &Line) -> String {
    // Every field serializes to JSON, so this never falls back.
    serde_json::to_string(line).unwrap_or_default()
}

/// Reads one line of the event log.
///
/// # Errors
///
/// Returns the JSON error when `text` is not a line of the log.
pub fn parse_line(text: &str) -> Result<Line, serde_json::Error> {
    serde_json::from_str(text)
}
