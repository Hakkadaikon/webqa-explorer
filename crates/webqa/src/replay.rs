//! A judge that replays the answers of a report.
//!
//! A replay goes through the same run loop and guardrails with the same
//! settings, so the questions come in the order they were recorded. The
//! questions are not compared with the recorded ones.

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, VecDeque};
use std::future;

use serde::Deserialize;
use webqa_core::event::Mode;
use webqa_core::judge::{Answers, JudgeError, Question, QuestionId, Usage};
use webqa_core::ports::{Judge, Pending};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;

use crate::store::report::SCHEMA;

/// A judge that gives back the replies of a report, in order.
#[derive(Debug, Clone, Default)]
pub struct Replay {
    replies: VecDeque<Result<Answers, JudgeError>>,
}

/// Why a report cannot be replayed. Carries nothing of the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("実行結果を再実行の台本として読めない")]
pub struct ReplayError;

/// The parts of `report.json` a replay reads.
#[derive(Deserialize)]
struct ReportFile {
    schema: u32,
    mode: Mode,
    calls: Vec<CallEntry>,
}

/// The answers of a call, by question; a missing probability is `NaN`.
type Table = BTreeMap<String, Vec<(String, Option<f64>)>>;

/// One judge call of the report.
#[derive(Deserialize)]
struct CallEntry {
    error: Option<JudgeError>,
    answers: Option<Table>,
    usage: Option<UsageEntry>,
}

/// The tokens of one call.
#[derive(Deserialize)]
struct UsageEntry {
    input_tokens: u64,
    output_tokens: u64,
}

impl Replay {
    /// The mode of the run and its replies, read from the text of `report.json`.
    ///
    /// # Errors
    ///
    /// [`ReplayError`] when the text is not a report of version 1.
    pub fn from_report(text: &str) -> Result<(Mode, Self), ReplayError> {
        let report: ReportFile = serde_json::from_str(text).or(Err(ReplayError))?;
        let current = Some(report).filter(|report| report.schema == SCHEMA);
        let report = current.ok_or(ReplayError)?;
        let replies = report.calls.into_iter().map(CallEntry::reply).collect();
        Ok((report.mode, Self { replies }))
    }
}

impl CallEntry {
    /// The error when there is one, the answers otherwise.
    fn reply(self) -> Result<Answers, JudgeError> {
        let Self {
            error,
            answers,
            usage,
        } = self;
        error.map_or_else(|| Ok(replied(answers, usage)), Err)
    }
}

/// The answers of a call that answered.
fn replied(answers: Option<Table>, usage: Option<UsageEntry>) -> Answers {
    let table = answers.unwrap_or_default().into_iter();
    Answers {
        table: table
            .map(|(id, options)| (QuestionId::new(&id), restored(options)))
            .collect(),
        usage: usage.map_or_else(Usage::default, UsageEntry::usage),
    }
}

/// The options with a missing probability, written for `NaN`, made `NaN` again.
fn restored(options: Vec<(String, Option<f64>)>) -> Vec<(String, f64)> {
    let restore = |(option, p): (String, Option<f64>)| (option, p.unwrap_or(f64::NAN));
    options.into_iter().map(restore).collect()
}

impl UsageEntry {
    const fn usage(self) -> Usage {
        Usage {
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
        }
    }
}

impl Judge for Replay {
    fn ask<'a>(
        &'a mut self,
        _screen: &'a Redacted<ScreenText>,
        _questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        let reply = self
            .replies
            .pop_front()
            .unwrap_or(Err(JudgeError::Exhausted));
        Box::pin(future::ready(reply))
    }
}
