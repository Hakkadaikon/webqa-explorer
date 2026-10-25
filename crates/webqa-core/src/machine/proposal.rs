//! The judge's answers read into a proposal.
//!
//! The answers are checked against what the step offered, and the element
//! is picked from the ranked numbers, passing over the elements operated
//! since the screen last changed.

use super::Seen;
use super::note::Anomaly;
use super::offer::{offer, verdict};
use super::ops::{OpRow, offered};
use super::state::State;
use crate::effect::Touch;
use crate::explore::ElementOp;
use crate::guard::identity::Target as Identity;
use crate::judge::{Answers, JudgeError, Verdict, pick_target};
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
    /// The number of the element in the screen text, from 1.
    pub(super) number: Option<usize>,
    /// The position of the element in `screen.elements`.
    pub(super) index: Option<usize>,
    /// The element as it was when the judge was asked.
    pub(super) identity: Option<Identity>,
    /// The probability of the chosen operation kind. `None` when missing.
    pub(super) confidence: Option<f64>,
}

/// An element inside the viewport: its number in the screen text, from 1,
/// and its position in `screen.elements`.
struct Numbered {
    number: usize,
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
            number: None,
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
    state: &State,
) -> Result<Proposal, Anomaly> {
    let answers = reply
        .as_ref()
        .map_err(|error| Anomaly::Judge(error.clone()))?;
    let verdict = verdict(answers, &offer(state)).map_err(Anomaly::Unusable)?;
    Ok(proposal(&verdict, state))
}

/// The operation kind chosen and, for an operation on an element, the
/// element picked from the ranked numbers.
fn proposal(verdict: &Verdict, state: &State) -> Proposal {
    let row = row_of(verdict, state);
    let pointed = row
        .and_then(|row| row.element)
        .and_then(|kind| pointed(verdict, state, kind));
    let number = pointed.as_ref().map(|p| p.number);
    let (element, index, identity) = located(pointed, state);
    Proposal {
        op: row.map(|row| row.name.to_owned()).unwrap_or_default(),
        row,
        element,
        number,
        index,
        identity,
        confidence: verdict.confidence,
    }
}

/// The element pointed at, its position in `screen.elements`, and its
/// reference when the judge was asked.
fn located(
    pointed: Option<Numbered>,
    state: &State,
) -> (Option<ElementId>, Option<usize>, Option<Identity>) {
    let index = pointed.as_ref().map(|p| p.index);
    let identity = index.and_then(|i| Identity::at(&state.last.material.nodes, i));
    (pointed.map(|p| p.element), index, identity)
}

/// The row of the operation kind chosen. The answers name an offered kind,
/// so it is always found.
fn row_of(verdict: &Verdict, state: &State) -> Option<&'static OpRow> {
    offered(&state.rules)
        .into_iter()
        .find(|row| row.op == verdict.op)
}

/// The element picked from the ranked numbers.
fn pointed(verdict: &Verdict, state: &State, kind: ElementOp) -> Option<Numbered> {
    let shown: Vec<Numbered> = numbered(&state.last).collect();
    let ranked = fields_or_all(&verdict.targets, state, &shown, kind);
    let number = pick_target(&ranked, &avoid(state))?;
    shown.into_iter().find(|e| e.number == number)
}

/// For `input`, only the ranked elements that take a value, since the
/// operation kind and the element are chosen apart. All of `ranked` for
/// another kind, or when none takes a value.
fn fields_or_all(
    ranked: &[usize],
    state: &State,
    shown: &[Numbered],
    kind: ElementOp,
) -> Vec<usize> {
    if kind != ElementOp::Input {
        return ranked.to_vec();
    }
    let fields: Vec<usize> = ranked
        .iter()
        .copied()
        .filter(|&number| takes_value(state, shown, number))
        .collect();
    if fields.is_empty() {
        ranked.to_vec()
    } else {
        fields
    }
}

/// Whether the element `number` among `shown` is a field with a value to type.
fn takes_value(state: &State, shown: &[Numbered], number: usize) -> bool {
    let screen = &state.last.screen;
    shown
        .iter()
        .find(|e| e.number == number)
        .is_some_and(|e| state.rules.input_skip(screen, Some(&e.element)).is_none())
}

/// The numbers of the elements operated since the screen last changed.
fn avoid(state: &State) -> Vec<usize> {
    numbered(&state.last)
        .filter(|e| state.unmoved.contains(&e.element))
        .map(|e| e.number)
        .collect()
}

/// The elements inside the viewport, numbered as in the screen text.
fn numbered(last: &Seen) -> impl Iterator<Item = Numbered> + '_ {
    let screen = last.screen.as_inner();
    element_ids(&last.screen)
        .into_iter()
        .zip(&screen.elements)
        .enumerate()
        .filter(|(_, (_, e))| in_viewport(e, last.viewport))
        .zip(1_usize..)
        .map(|((index, (element, _)), number)| Numbered {
            number,
            index,
            element,
        })
}
