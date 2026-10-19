//! The judge's answers read into a proposal.
//!
//! The answers are checked against what the step offered, and the element
//! is picked from the ranked numbers, passing over the element of the last
//! operation when the screen did not change after it.

use super::Seen;
use super::note::Anomaly;
use super::offer::{offer, verdict};
use super::ops::{OpRow, offered};
use super::state::State;
use crate::effect::Touch;
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
        .and_then(|_| pointed(verdict, state));
    let (element, index, identity) = located(pointed, state);
    Proposal {
        op: row.map(|row| row.name.to_owned()).unwrap_or_default(),
        row,
        element,
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
fn pointed(verdict: &Verdict, state: &State) -> Option<Numbered> {
    pick_target(&verdict.targets, avoid(state))
        .and_then(|number| numbered(&state.last).find(|e| e.number == number))
}

/// The number of the element of the last operation, when the screen did
/// not change after it.
fn avoid(state: &State) -> Option<usize> {
    let unmoved = state.unmoved.as_ref()?;
    numbered(&state.last)
        .find(|e| e.element == *unmoved)
        .map(|e| e.number)
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
