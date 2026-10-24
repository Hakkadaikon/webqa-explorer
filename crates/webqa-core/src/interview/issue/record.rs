//! What the run loop records of the trials, as the issues read it.

use super::{Act, Target};
use crate::config::raw::Op;
use crate::interview::answer::Reply;
use crate::interview::ask::Visit;
use crate::interview::clue::Clue;
use crate::interview::outcome::Outcome;
use crate::redact::Redacted;
use crate::screen::{Element, ElementId, Screen, Viewport, element_ids, in_viewport};

/// What ran on the page at a step, after the guardrails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ran {
    /// The operation kind.
    pub op: Op,
    /// The element's number in the step's screen text, from 1, for
    /// operations on an element.
    pub target: Option<usize>,
}

/// A step of a trial, as the issues read it.
#[derive(Debug, Clone, Copy)]
pub struct Seen<'a> {
    /// The step number, as the event log numbers it.
    pub step: u64,
    /// The screen the judge was asked about.
    pub screen: &'a Redacted<Screen>,
    /// The viewport the screen text was made with.
    pub viewport: Viewport,
    /// The checked answers. `None` when they were unusable.
    pub reply: Option<&'a Reply>,
    /// The clues of the step from [`clues`](crate::interview::clue::clues).
    /// Read only when `reply` is `Some`.
    pub clues: &'a [Clue],
    /// What ran on the page, if anything did.
    pub ran: Option<Ran>,
}

/// A trial, as the issues read it.
#[derive(Debug, Clone, Copy)]
pub struct Record<'a> {
    /// How it ended. Only finished trials count.
    pub outcome: Outcome,
    /// Its steps in order.
    pub steps: &'a [Seen<'a>],
}

/// The trials of a run.
#[derive(Debug, Clone, Copy)]
pub struct Run<'a> {
    /// Every trial started, in order. The trial numbers are positions
    /// in this slice, from 1.
    pub trials: &'a [Record<'a>],
    /// Whether the judge declared that it reads images.
    pub images: bool,
}

/// The steps of `record` as the mismatch questions read them.
#[must_use]
pub fn visits<'a>(record: &Record<'a>) -> Vec<Visit<'a>> {
    record.steps.iter().map(visit).collect()
}

fn visit<'a>(seen: &Seen<'a>) -> Visit<'a> {
    Visit {
        url: &seen.screen.as_inner().url,
        guess: seen.reply.map(|reply| reply.purpose_guess.as_str()),
    }
}

/// The element numbered `n` (from 1) in the screen text of `seen`: its
/// identity and its target. `None` when no element has that number.
pub(super) fn element(seen: &Seen<'_>, n: usize) -> Option<(ElementId, Target)> {
    let screen = seen.screen.as_inner();
    element_ids(seen.screen)
        .into_iter()
        .zip(&screen.elements)
        .filter(|(_, e)| in_viewport(e, seen.viewport))
        .nth(n.checked_sub(1)?)
        .map(|(id, e)| target(id, e))
}

/// The identity of `element` with its target.
fn target(id: ElementId, element: &Element) -> (ElementId, Target) {
    let target = Target {
        role: element.role.clone(),
        name: element.name.clone(),
        nth: id.nth,
        rect: element.rect,
    };
    (id, target)
}

/// What ran on the page at the steps before step `step` (from 1).
pub(super) fn path(steps: &[Seen<'_>], step: usize) -> Vec<Act> {
    steps
        .iter()
        .take(step.saturating_sub(1))
        .filter_map(act)
        .collect()
}

/// What ran on the page at `seen`, if anything did.
fn act(seen: &Seen<'_>) -> Option<Act> {
    seen.ran.map(|ran| Act {
        op: ran.op,
        target: ran.target.and_then(|n| element(seen, n)).map(|(_, t)| t),
    })
}
