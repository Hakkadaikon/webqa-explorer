//! Where the hesitation, visibility and unreached issues happen in one
//! trial.

use super::record::{element, path};
use super::{Act, Detail, Halt, Record, Seen, Shot, Target};
use crate::interview::answer::{HardToSee, Reply};
use crate::interview::outcome::{Ending, Stuck};
use crate::screen::{ElementId, ScreenId, screen_id};

/// One occurrence of an issue in a trial.
pub(super) struct Found {
    /// The identity of the screen it happened at.
    pub(super) screen: ScreenId,
    /// The identity of its element, if it has one.
    pub(super) element: Option<ElementId>,
    /// The URL of the screen.
    pub(super) url: String,
    /// Its element, if it has one.
    pub(super) target: Option<Target>,
    /// What the judge said.
    pub(super) detail: Detail,
    /// Where in the run it happened.
    pub(super) shot: Shot,
    /// What ran on the page before it, in its trial.
    pub(super) path: Vec<Act>,
    /// The confidence of the operation chosen there.
    pub(super) confidence: Option<f64>,
}

impl Found {
    /// Whether `self` and `other` have the same key: screen, element and
    /// kind.
    pub(super) fn same_key(&self, other: &Self) -> bool {
        (&self.screen, &self.element, self.detail.kind())
            == (&other.screen, &other.element, other.detail.kind())
    }
}

/// A trial and where it is in the run.
#[derive(Clone, Copy)]
pub(super) struct At<'r> {
    /// The trial.
    pub(super) record: &'r Record<'r>,
    /// Its number among the trials started, from 1.
    pub(super) trial: usize,
    /// Whether the judge reads images.
    pub(super) images: bool,
    /// How it ended.
    pub(super) ending: Ending,
}

/// A step of a trial: the step and its number, from 1.
#[derive(Clone, Copy)]
struct Step<'r> {
    at: At<'r>,
    seen: &'r Seen<'r>,
    number: usize,
}

/// The occurrences in the trial, step by step, then the unreached one.
pub(super) fn found(at: At<'_>) -> Vec<Found> {
    let steps = at
        .record
        .steps
        .iter()
        .zip(1..)
        .map(|(seen, number)| Step { at, seen, number });
    let mut found: Vec<Found> = steps.flat_map(step_found).collect();
    found.extend(unreached(at));
    found
}

/// The hesitation and the visibility issues of a step with usable answers.
fn step_found(step: Step<'_>) -> Vec<Found> {
    step.seen
        .reply
        .map(|reply| {
            let mut found: Vec<Found> = hesitant(step, reply).into_iter().collect();
            found.extend(visibility(step, reply));
            found
        })
        .unwrap_or_default()
}

/// The hesitation at the step, on the element the judge chose.
fn hesitant(step: Step<'_>, reply: &Reply) -> Option<Found> {
    let clues = step.seen.clues;
    (!clues.is_empty()).then(|| {
        let chosen = reply.verdict.targets.first();
        let detail = Detail::Hesitant {
            clues: clues.to_vec(),
            expectation: reply.expectation.clone(),
        };
        let element = chosen.and_then(|&n| element(step.seen, n));
        occurrence(step, element, detail, reply.verdict.confidence)
    })
}

/// The elements listed as hard to see at the step, in the order listed,
/// when the judge reads images.
fn visibility(step: Step<'_>, reply: &Reply) -> Vec<Found> {
    let listed = step
        .at
        .images
        .then_some(reply.hard_to_see.as_deref())
        .flatten()
        .unwrap_or_default();
    listed
        .iter()
        .filter_map(|hard| hard_to_see(step, reply, hard))
        .collect()
}

/// The visibility issue of `hard`, unless its number names no element.
fn hard_to_see(step: Step<'_>, reply: &Reply, hard: &HardToSee) -> Option<Found> {
    element(step.seen, hard.element).map(|element| {
        let detail = Detail::Visibility {
            reason: hard.reason.clone(),
            expectation: reply.expectation.clone(),
        };
        occurrence(step, Some(element), detail, None)
    })
}

/// The unreached issue of the trial, at its last step.
fn unreached(at: At<'_>) -> Option<Found> {
    let halt = halt(at.ending)?;
    let (seen, number) = at.record.steps.iter().zip(1..).last()?;
    let detail = Detail::Unreached {
        halt,
        expectation: seen.reply.map(|reply| reply.expectation.clone()),
    };
    Some(occurrence(Step { at, seen, number }, None, detail, None))
}

/// How a finished trial that did not reach the purpose stopped. `None`
/// for the other finished trials.
const fn halt(ending: Ending) -> Option<Halt> {
    match ending {
        Ending::Stuck(Stuck::GaveUp) => Some(Halt::GaveUp),
        Ending::Stuck(Stuck::NoProgress) => Some(Halt::NoProgress),
        Ending::BudgetSpent => Some(Halt::BudgetSpent),
        Ending::Goal | Ending::Claimed(_) => None,
    }
}

/// An occurrence at `step`.
fn occurrence(
    step: Step<'_>,
    element: Option<(ElementId, Target)>,
    detail: Detail,
    confidence: Option<f64>,
) -> Found {
    let (element, target) = element.unzip();
    Found {
        screen: screen_id(step.seen.screen),
        element,
        url: step.seen.screen.as_inner().url.to_string(),
        target,
        detail,
        shot: Shot {
            trial: step.at.trial,
            step: step.number,
        },
        path: path(step.at.record.steps, step.number),
        confidence,
    }
}

#[cfg(test)]
mod tests;
