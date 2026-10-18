//! The judge's answers read into a proposal.
//!
//! A minimal reading until issue #7 checks the answers: the first option
//! of each question, as the judge ranks them.

use super::Seen;
use super::note::Anomaly;
use super::ops::{OpRow, offered};
use super::rules::Rules;
use crate::effect::Touch;
use crate::guard::identity::Target as Identity;
use crate::judge::{Answers, JudgeError, QuestionId};
use crate::screen::{ElementId, element_ids, in_viewport};

/// What to do, from the judge's answers or from leaving a paywall.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Proposal {
    /// The name of the operation, as the browser knows it.
    pub(super) op: String,
    /// The operation kind the judge chose. `None` when leaving a paywall.
    pub(super) row: Option<&'static OpRow>,
    /// The element to operate on.
    pub(super) element: Option<ElementId>,
    /// The position of the element in `screen.elements`.
    pub(super) index: Option<usize>,
    /// The element as it was when the judge was asked.
    pub(super) identity: Option<Identity>,
    /// The probability of the chosen operation kind. `None` when missing.
    pub(super) confidence: Option<f64>,
}

/// The element the judge pointed at.
struct Pointed {
    index: usize,
    element: ElementId,
}

impl Proposal {
    /// Leaving a paywall with the operation `op`.
    pub(super) const fn leave(op: String) -> Self {
        Self {
            op,
            row: None,
            element: None,
            index: None,
            identity: None,
            confidence: None,
        }
    }

    /// The browser operation. A missing confidence is 0.
    pub(super) fn touch(&self) -> Touch {
        Touch {
            op: self.op.clone(),
            target: self.element.clone(),
            confidence: self.confidence.unwrap_or(0.0),
        }
    }
}

/// The proposal of the judge's reply, or what was wrong with it.
pub(super) fn hear(
    reply: &Result<Answers, JudgeError>,
    rules: &Rules,
    last: &Seen,
) -> Result<Proposal, Anomaly> {
    reply
        .as_ref()
        .map_err(|error| Anomaly::Judge(error.clone()))
        .and_then(|answers| read(answers, rules, last))
}

/// Reads the operation kind among those offered and, for an operation on
/// an element, the element by its number in the screen text of `last`.
fn read(answers: &Answers, rules: &Rules, last: &Seen) -> Result<Proposal, Anomaly> {
    let (row, confidence) = chosen(answers, rules).ok_or(Anomaly::Operation)?;
    let pointed = pointed(row, answers, last)?;
    Ok(proposal(row, confidence, pointed, last))
}

/// The offered operation kind the judge chose, with its probability.
fn chosen(answers: &Answers, rules: &Rules) -> Option<(&'static OpRow, Option<f64>)> {
    let (name, confidence) = operation(answers)?;
    let row = offered(rules).into_iter().find(|row| row.name == name)?;
    Some((row, confidence))
}

/// The element, for an operation on one; nothing for the others.
fn pointed(row: &OpRow, answers: &Answers, last: &Seen) -> Result<Option<Pointed>, Anomaly> {
    row.element
        .map(|_| target(answers, last).ok_or(Anomaly::Target))
        .transpose()
}

fn proposal(
    row: &'static OpRow,
    confidence: Option<f64>,
    pointed: Option<Pointed>,
    last: &Seen,
) -> Proposal {
    let index = pointed.as_ref().map(|p| p.index);
    Proposal {
        op: row.name.to_owned(),
        row: Some(row),
        element: pointed.map(|p| p.element),
        index,
        identity: index.and_then(|i| Identity::at(&last.material.nodes, i)),
        confidence,
    }
}

/// The most probable operation kind; the first one with a missing
/// probability when every probability is missing.
fn operation(answers: &Answers) -> Option<(&str, Option<f64>)> {
    let id = QuestionId::operation();
    answers
        .best(&id)
        .map(|(name, p)| (name, Some(p)))
        .or_else(|| first(answers, &id))
}

fn first<'a>(answers: &'a Answers, id: &QuestionId) -> Option<(&'a str, Option<f64>)> {
    let (name, _) = answers.table.get(id).and_then(|options| options.first())?;
    Some((name.as_str(), None))
}

/// The element whose number in the screen text is the most probable answer.
fn target(answers: &Answers, last: &Seen) -> Option<Pointed> {
    let (choice, _) = answers.best(&QuestionId::target())?;
    let screen = last.screen.as_inner();
    element_ids(&last.screen)
        .into_iter()
        .zip(&screen.elements)
        .enumerate()
        .filter(|(_, (_, e))| in_viewport(e, last.viewport))
        .zip(1_usize..)
        .find(|(_, number)| number.to_string() == choice)
        .map(|((index, (element, _)), _)| Pointed { index, element })
}
