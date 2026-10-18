//! Named conditions of the rows: the phase, the kind of facts, and the
//! checks that stop a proposal.

use super::material::forbidden_indices;
use super::note::Rejection;
use super::proposal::Proposal;
use super::state::{Phase, State};
use super::{Facts, Heard, Seen};
use crate::config::raw::Op;
use crate::guard::confidence::is_low;
use crate::judge::{Answers, JudgeError};

/// The first tier of waiting does not hold: still exploring.
///
/// Only the phase for now; the goal of the stage comes with a later issue.
pub(super) fn exploring(state: &State) -> bool {
    state.phase == Phase::Idle
}

/// The page, if the facts are an observation.
pub(super) fn seen(facts: &Facts) -> Option<&Seen> {
    match facts {
        Facts::Page(seen) => Some(seen.as_ref()),
        Facts::Outside(_) => None,
    }
}

/// The judge's reply, if the facts are one.
pub(super) const fn answered(facts: &Facts) -> Option<&Result<Answers, JudgeError>> {
    match facts {
        Facts::Outside(Heard::Answered(reply)) => Some(reply),
        Facts::Outside(Heard::Woke) | Facts::Page(_) => None,
    }
}

/// A check that stops a proposal, and what it records.
struct Check {
    stops: fn(&State, &Proposal) -> bool,
    why: Rejection,
}

/// The checks, in order.
static CHECKS: [Check; 3] = [
    Check {
        stops: forbidden,
        why: Rejection::Forbidden,
    },
    Check {
        stops: unidentified,
        why: Rejection::Missing,
    },
    Check {
        stops: doubtful_back,
        why: Rejection::LowConfidence,
    },
];

/// Why the guardrails stop the proposal, if they do.
pub(super) fn rejection(state: &State, proposal: &Proposal) -> Option<Rejection> {
    CHECKS
        .iter()
        .find(|check| (check.stops)(state, proposal))
        .map(|check| check.why)
}

/// The element is forbidden on the page the judge saw.
fn forbidden(state: &State, proposal: &Proposal) -> bool {
    proposal
        .index
        .is_some_and(|i| forbidden_indices(&state.rules, &state.last).contains(&i))
}

/// The element has no reference to look it up by.
const fn unidentified(_: &State, proposal: &Proposal) -> bool {
    proposal.element.is_some() && proposal.identity.is_none()
}

/// A "back" the judge is not sure enough of.
fn doubtful_back(state: &State, proposal: &Proposal) -> bool {
    proposal.row.is_some_and(|row| row.op == Op::Back)
        && is_low(proposal.confidence, state.rules.back_confidence)
}
