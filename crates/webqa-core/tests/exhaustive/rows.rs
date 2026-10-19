//! What the search must reach, so that no property holds for want of cases.

use webqa_core::issue::{Issue, IssueKind};
use webqa_core::judge::{JudgeError, QuestionId, Unusable};
use webqa_core::machine::{Anomaly, ExitReason, Note, Phase, Rejection, StuckKind};
use webqa_core::redact::Redactor;

use super::candidates::PRICING;
use super::candidates::pages::AWAY_URL;
use super::view::{Transition, all, asks, looks, sleeps};

/// Something to reach, by name.
pub struct Row {
    /// What is reached.
    pub name: String,
    /// Whether the transition reaches it.
    pub reached: Box<dyn Fn(&Transition) -> bool + Send + Sync>,
}

fn row(name: &str, reached: impl Fn(&Transition) -> bool + Send + Sync + 'static) -> Row {
    Row {
        name: name.to_owned(),
        reached: Box::new(reached),
    }
}

/// The effect `asked` asks for.
fn effect(name: &str, asked: fn(&Transition) -> bool) -> Row {
    row(name, asked)
}

/// The effects, the notes and the reasons the search must reach.
pub fn rows() -> Vec<Row> {
    effects()
        .into_iter()
        .chain(notes().into_iter().map(noted))
        .chain(REASONS.into_iter().map(ended))
        .collect()
}

fn effects() -> Vec<Row> {
    vec![
        effect("ask", asks),
        effect("look up", looks),
        effect("sleep", sleeps),
        row("operate on an element", |t| t.target().is_some()),
        row("leave the paywall", |t| {
            let own = t.before.phase() != Phase::Awaiting;
            all(&[t.touches("back"), t.last == Some(PRICING), own])
        }),
        row("reload", |t| t.touches("reload")),
        row("return", |t| t.touches("return")),
    ]
}

/// The rejections, landing outside, and the anomalies of the answers.
fn notes() -> Vec<Note> {
    let left = Issue {
        kind: IssueKind::OutsideDomain,
        evidence: Some(Redactor::default().text(AWAY_URL)),
        label: None,
        p: None,
    };
    vec![
        Note::Rejected(Rejection::Forbidden),
        Note::Rejected(Rejection::LowConfidence),
        Note::Rejected(Rejection::Missing),
        Note::Rejected(Rejection::Replaced),
        Note::Issue(left),
        Note::Anomaly(Anomaly::Judge(JudgeError::Exhausted)),
        not_an_option(QuestionId::operation()),
        not_an_option(QuestionId::target()),
    ]
}

/// The anomaly of an option of the question `id` that was not asked.
const fn not_an_option(id: QuestionId) -> Note {
    Note::Anomaly(Anomaly::Unusable(Unusable::NotAnOption(id)))
}

const REASONS: [ExitReason; 5] = [
    ExitReason::Stuck(StuckKind::NoProgress),
    ExitReason::Stuck(StuckKind::Declared),
    ExitReason::BudgetExceeded,
    ExitReason::TimeExceeded,
    ExitReason::ReportedDone,
];

fn noted(note: Note) -> Row {
    row(&format!("{note:?}"), move |t| t.adds(&note))
}

fn ended(reason: ExitReason) -> Row {
    row(&format!("{reason:?}"), move |t| t.reason() == Some(reason))
}
