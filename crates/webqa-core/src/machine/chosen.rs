//! The operation and element a step of an interview is about to carry out,
//! chosen the way the core chooses them.

use super::guards::rejection;
use super::note::Rejection;
use super::proposal::{Proposal, hear};
use super::state::State;
use crate::effect::Touch;
use crate::judge::{Answers, JudgeError};

#[cfg(test)]
mod tests;

/// The element chosen: its number in the screen text, from 1, and its
/// name as the judge saw it (redacted).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pointed {
    /// The number in the screen text.
    pub number: usize,
    /// The accessible name, redacted.
    pub name: String,
}

/// An operation the core will carry out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chosen {
    /// The name of the operation kind, as the browser knows it.
    pub op: &'static str,
    /// The element, for an operation on one.
    pub element: Option<Pointed>,
}

/// What the core does with `reply` in `state`, when it carries out an
/// operation. `None` when the reply is unusable, the guardrails stop it,
/// or it ends the run (done, stuck).
#[must_use]
pub fn chosen(reply: &Result<Answers, JudgeError>, state: &State) -> Option<Chosen> {
    let proposal = hear(reply, state).ok()?;
    let row = proposal.row.filter(|row| row.ends.is_none())?;
    let stopped = rejection(state, &proposal).is_some();
    (!stopped).then(|| Chosen {
        op: row.name,
        element: pointed(proposal),
    })
}

/// The operation the guardrails stop on `reply` in `state`, and why: the
/// other side of [`chosen`]. `None` when the reply is unusable or nothing
/// stops it.
#[must_use]
pub fn stopped(reply: &Result<Answers, JudgeError>, state: &State) -> Option<(Touch, Rejection)> {
    let proposal = hear(reply, state).ok()?;
    rejection(state, &proposal).map(|why| (proposal.touch(), why))
}

/// The element of the proposal, with its number.
fn pointed(proposal: Proposal) -> Option<Pointed> {
    let (element, number) = proposal.element.zip(proposal.number)?;
    Some(Pointed {
        number,
        name: element.name,
    })
}
