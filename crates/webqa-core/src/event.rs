//! The event log: one JSON object per line, shared by the CLI and the TUI.

use serde::{Deserialize, Serialize};

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
    /// The judge chose an operation.
    Decided {
        /// The operation.
        op: String,
        /// The target element, if the operation needs one.
        target: Option<String>,
        /// Confidence of the choice.
        p: f64,
    },
    /// The browser ran the operation.
    Executed {
        /// Whether the screen changed.
        changed: bool,
        /// Number of network accesses.
        accesses: u32,
    },
    /// The last line of a run.
    RunFinished {
        /// Why the run ended.
        reason: String,
    },
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

#[cfg(test)]
mod tests {
    use super::*;

    const DOCUMENTED: [&str; 4] = [
        r#"{"seq":0,"elapsed_ms":0,"kind":"run_started","schema":1,"mode":"explore","started_at":"2026-10-18T02:00:00Z"}"#,
        r#"{"seq":1,"elapsed_ms":1200,"step":1,"kind":"decided","op":"click","target":"保存","p":0.9}"#,
        r#"{"seq":2,"elapsed_ms":2100,"step":1,"kind":"executed","changed":true,"accesses":1}"#,
        r#"{"seq":9,"elapsed_ms":60000,"kind":"run_finished","reason":"goal_achieved"}"#,
    ];

    #[test]
    fn mode_name_matches_the_log() {
        for (mode, name) in [(Mode::Explore, "explore"), (Mode::Scenario, "scenario")] {
            assert_eq!(
                mode.as_str(),
                name,
                "the name is the one written to the log"
            );
            assert_eq!(
                serde_json::to_string(&mode).unwrap(),
                format!("\"{name}\""),
                "serde writes the same name"
            );
        }
    }

    #[test]
    fn event_lines_match_documented_jsonl() {
        for text in DOCUMENTED {
            let line = parse_line(text).unwrap();
            assert_eq!(to_json_line(&line), text, "the line round-trips unchanged");
        }
        assert_eq!(
            parse_line(DOCUMENTED[3]).unwrap(),
            Line {
                seq: 9,
                elapsed_ms: 60000,
                step: None,
                event: Event::RunFinished {
                    reason: "goal_achieved".to_owned()
                },
            },
            "the fields are read into the line"
        );
    }
}
