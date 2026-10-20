//! What a run did, kept for its report: the judge calls, the browser
//! operations and the issues found.

#[cfg(test)]
mod tests;

use std::time::Duration;

use webqa_core::interview::ask::{Past, recent_section, with_section};
use webqa_core::issue::{Issue, has_confirmed};
use webqa_core::judge::{Answers, JudgeError, Usage};
use webqa_core::machine::Cause;
use webqa_core::redact::{Redacted, Redactor};
use webqa_core::screen::{ElementId, ScreenText};

/// What a run did, for the report.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Record {
    /// Each judge call, in order.
    pub calls: Vec<Call>,
    /// Each browser operation, in order.
    pub acts: Vec<Act>,
    /// Each issue found, once each, in order.
    pub found: Vec<Found>,
}

/// One judge call.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    /// The step it belongs to, from 1.
    pub step: u64,
    /// The answers, or why there were none.
    pub reply: Result<Answers, JudgeError>,
    /// How long the judge took.
    pub think: Duration,
}

/// One browser operation.
#[derive(Debug, Clone, PartialEq)]
pub struct Act {
    /// The step it belongs to.
    pub step: u64,
    /// The operation, as the browser knows it.
    pub op: String,
    /// The element operated on, if any. Its name is redacted.
    pub target: Option<ElementId>,
    /// The judge's probability, 0 when missing; 1 for the core's own operations.
    pub p: f64,
    /// Who asked for it.
    pub cause: Cause,
    /// Whether the screen changed by it: the URL or the viewport's text differs.
    pub changed: bool,
    /// How long the operation took.
    pub act: Duration,
    /// How long the page took to be observed after it.
    pub settle: Duration,
}

/// One issue and the screenshot saved for it.
#[derive(Debug, Clone, PartialEq)]
pub struct Found {
    /// The step it was found at.
    pub step: u64,
    /// The issue.
    pub issue: Issue,
    /// The file name of the screenshot, under `screenshots/`; none when it
    /// could not be taken.
    pub shot: Option<String>,
}

impl Record {
    /// Tokens used by the calls that answered.
    #[must_use]
    pub fn usage(&self) -> Usage {
        self.calls
            .iter()
            .filter_map(|call| call.reply.as_ref().ok())
            .fold(Usage::default(), |total, answers| add(total, answers.usage))
    }

    /// `screen` and the latest operations from the `from`th on, the section
    /// passed through `redactor`.
    #[must_use]
    pub fn told(
        &self,
        from: usize,
        screen: &Redacted<ScreenText>,
        redactor: &Redactor,
    ) -> Redacted<ScreenText> {
        let section = recent_section(&self.past(from));
        let with = |text: String| with_section(screen, &redactor.text(&text));
        section.map_or_else(|| screen.clone(), with)
    }

    fn past(&self, from: usize) -> Vec<Past> {
        let acts = self.acts.get(from..).unwrap_or_default();
        let past = acts.iter().map(|act| Past {
            op: act.op.clone(),
            target: act.target.clone(),
            changed: act.changed,
        });
        past.collect()
    }

    /// Whether a confirmed issue was found.
    #[must_use]
    pub fn confirmed(&self) -> bool {
        let issues: Vec<Issue> = self.found.iter().map(|f| f.issue.clone()).collect();
        has_confirmed(&issues)
    }

    /// Whether an issue of the same kind, evidence and label is recorded.
    #[must_use]
    pub fn knows(&self, issue: &Issue) -> bool {
        self.found.iter().any(|f| same(&f.issue, issue))
    }
}

const fn add(total: Usage, usage: Usage) -> Usage {
    Usage {
        input_tokens: total.input_tokens.saturating_add(usage.input_tokens),
        output_tokens: total.output_tokens.saturating_add(usage.output_tokens),
    }
}

/// The same kind, evidence and label; the probability is not compared.
fn same(a: &Issue, b: &Issue) -> bool {
    (a.kind, &a.evidence, &a.label) == (b.kind, &b.evidence, &b.label)
}
