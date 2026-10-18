//! The rows of the table from waiting for the judge on, and what the rows share.

#[cfg(test)]
mod answer_tests;
#[cfg(test)]
mod page_tests;
#[cfg(test)]
mod stop_tests;
#[cfg(test)]
mod tests;

use std::time::Duration;

use super::guards::{answered, observing, rejection, seen, woke};
use super::note::{Anomaly, Note, Rejection};
use super::observe::{arrived, executed, take};
use super::ops::names;
use super::proposal::{Proposal, hear};
use super::state::{ExitReason, Phase, State};
use super::{Facts, Outcome, Seen, Step};
use crate::effect::{Effect, PageEffect, Questions, Target, Touch};
use crate::guard::identity::{Recheck, recheck};
use crate::judge::step_questions;
use crate::screen::{screen_text, visible};

/// The facts of the last effect and the time since the run started.
pub(super) struct Now {
    pub(super) facts: Facts,
    pub(super) at: Duration,
}

/// What a row does.
pub(super) type Fire = fn(State, &Now) -> Next;

/// One row: when it applies and what it does.
pub(super) struct Action {
    pub(super) enabled: fn(&State, &Now) -> bool,
    pub(super) fire: Fire,
}

/// What a row did.
pub(super) enum Next {
    /// Run the table again with the same facts.
    Go(Box<State>),
    /// Return this to the run loop.
    Out(Step),
}

/// The first row: an ended run stays ended.
pub(super) static ENDED: [Action; 1] = [Action {
    enabled: |state, _| state.ended().is_some(),
    fire: |state, _| {
        let reason = state.ended().unwrap_or(ExitReason::UserAborted);
        Next::Out(Step::Finished(Outcome {
            reason,
            state: Box::new(state),
        }))
    },
}];

/// The rows that take the result of an effect.
pub(super) static ROWS: [Action; 6] = [
    Action {
        enabled: |state, now| state.phase == Phase::Awaiting && answered(&now.facts).is_some(),
        fire: decide,
    },
    Action {
        enabled: |state, now| observing(state, &now.facts, Phase::Confirming),
        fire: confirm,
    },
    Action {
        enabled: |state, now| observing(state, &now.facts, Phase::Acting),
        fire: executed,
    },
    Action {
        enabled: |state, now| observing(state, &now.facts, Phase::Returning),
        fire: arrived,
    },
    Action {
        enabled: |state, now| observing(state, &now.facts, Phase::Recovering),
        fire: arrived,
    },
    Action {
        enabled: |state, now| state.phase == Phase::Sleeping && woke(&now.facts),
        fire: |state, _| idle(state),
    },
];

/// The questions about the latest page.
pub(super) fn questions(state: &State) -> Questions {
    let last = &state.last;
    let elements = visible(last.screen.as_inner(), last.viewport).len();
    Questions {
        screen: screen_text(&last.screen, last.viewport),
        questions: step_questions(&names(&state.rules), elements),
    }
}

/// Back to waiting, with no proposal.
pub(super) fn idle(mut state: State) -> Next {
    state.proposal = None;
    Next::Go(Box::new(state.to(Phase::Idle)))
}

/// Facts that do not fit the phase: back to waiting, changing nothing else.
pub(super) const MISMATCH: Fire = |state, _| idle(state);

/// Ends the run for `reason`.
pub(super) fn finish(state: State, reason: ExitReason) -> Next {
    Next::Out(Step::Finished(Outcome {
        reason,
        state: Box::new(state.to(Phase::Ended(reason))),
    }))
}

/// Asks the run loop for `effect`, waiting in `phase`.
pub(super) fn out(state: State, phase: Phase, effect: Effect) -> Next {
    Next::Out(Step::Continue(Box::new(state.to(phase)), effect))
}

/// Asks for the browser operation `touch`, counting it in the time window.
pub(super) fn touched(mut state: State, phase: Phase, touch: Touch, now: Duration) -> Next {
    state.window = state.window.spend(now);
    out(state, phase, Effect::Page(PageEffect::Touch(touch)))
}

/// The page in the facts, or the latest one if there is none.
pub(super) fn page(state: &State, now: &Now) -> Seen {
    seen(&now.facts).unwrap_or(&state.last).clone()
}

/// Takes the judge's reply: a proposal to stop, finish with, or pass on.
fn decide(state: State, now: &Now) -> Next {
    let proposal = answered(&now.facts).map_or(Err(Anomaly::Operation), |reply| {
        hear(reply, &state.rules, &state.last)
    });
    match proposal {
        Ok(proposal) => judged(state, proposal, now.at),
        Err(anomaly) => idle(state.noted(Note::Anomaly(anomaly))),
    }
}

/// Stops the proposal, or else finishes with it or passes it on.
fn judged(state: State, proposal: Proposal, now: Duration) -> Next {
    match rejection(&state, &proposal) {
        Some(why) => idle(state.noted(Note::Rejected(why))),
        None => pass(state, proposal, now),
    }
}

/// An operation that ends the run, or one to carry out.
fn pass(state: State, proposal: Proposal, now: Duration) -> Next {
    match proposal.row.and_then(|row| row.ends) {
        Some(reason) => finish(state, reason),
        None => carry_out(state, proposal, now),
    }
}

/// Looks the element up again first, or operates at once without one.
fn carry_out(mut state: State, proposal: Proposal, now: Duration) -> Next {
    let element = proposal.element.clone();
    let touch = proposal.touch();
    state.proposal = Some(Box::new(proposal));
    match element {
        Some(element) => out(
            state,
            Phase::Confirming,
            Effect::Page(PageEffect::Look(Target { element })),
        ),
        None => touched(state, Phase::Acting, touch, now),
    }
}

/// The element looked up again: operate on it if it is the same node.
fn confirm(state: State, now: &Now) -> Next {
    let seen = page(&state, now);
    let checked = state
        .proposal
        .as_deref()
        .map_or(Err(Rejection::Missing), |p| verdict(p, &seen));
    match checked {
        Ok(touch) => touched(state, Phase::Acting, touch, now.at),
        Err(why) => idle(take(state.noted(Note::Rejected(why)), seen).0),
    }
}

/// The operation, if the element is the same node as when the judge was asked.
fn verdict(proposal: &Proposal, seen: &Seen) -> Result<Touch, Rejection> {
    let check = proposal
        .identity
        .as_ref()
        .map_or(Recheck::Missing, |identity| {
            recheck(identity, &seen.material.nodes)
        });
    match check {
        Recheck::Same => Ok(proposal.touch()),
        Recheck::Missing => Err(Rejection::Missing),
        Recheck::Replaced => Err(Rejection::Replaced),
    }
}
