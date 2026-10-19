//! Issues read from the judge's answers: the error display, and the
//! semantic findings the judge is asked about each step.

#[cfg(test)]
mod tests;

use super::{Finding, Issue, error_display_issue, semantic_issues};
use crate::judge::{Answers, Question, QuestionId};

/// A semantic finding the judge is asked about each step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
    /// The question's name.
    pub id: QuestionId,
    /// The fixed name of the finding written to the records.
    pub label: String,
    /// The question.
    pub prompt: String,
}

/// The findings asked about in a real run.
#[must_use]
pub fn probes() -> Vec<Probe> {
    vec![Probe {
        id: QuestionId::new("confusing_display"),
        label: "わかりにくい表示".to_owned(),
        prompt: "画面にわかりにくい表示があるか".to_owned(),
    }]
}

/// One yes-or-no question per probe, in order.
#[must_use]
pub fn probe_questions(probes: &[Probe]) -> Vec<Question> {
    probes.iter().map(question).collect()
}

fn question(probe: &Probe) -> Question {
    Question::Noul {
        id: probe.id.clone(),
        prompt: probe.prompt.clone(),
    }
}

/// The issues the answers show: the error display first, then the
/// semantic findings in the order of `probes`.
///
/// Each reaches `error_threshold` or is left out. The answers are read on
/// their own, whether or not the rest of the reply could be used.
#[must_use]
pub fn answered_issues(answers: &Answers, error_threshold: f64, probes: &[Probe]) -> Vec<Issue> {
    let error = yes(answers, &QuestionId::error_shown());
    let findings: Vec<Finding> = probes.iter().map(|p| finding(answers, p)).collect();
    error_display_issue(error, error_threshold)
        .into_iter()
        .chain(semantic_issues(&findings, error_threshold))
        .collect()
}

fn finding(answers: &Answers, probe: &Probe) -> Finding {
    Finding {
        label: probe.label.clone(),
        p: yes(answers, &probe.id),
    }
}

/// The probability of the option `"true"` of the question `id`. `None`
/// when there is no answer or no such option. A `NaN` never reaches a
/// threshold, so it records nothing.
fn yes(answers: &Answers, id: &QuestionId) -> Option<f64> {
    answers
        .table
        .get(id)?
        .iter()
        .find(|(option, _)| option == "true")
        .map(|(_, p)| *p)
}
