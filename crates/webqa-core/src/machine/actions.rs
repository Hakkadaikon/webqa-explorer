//! The rows of the table from waiting for the judge on, and what the rows share.

#[cfg(test)]
mod answer_tests;
#[cfg(test)]
mod explore_tests;
#[cfg(test)]
mod page_tests;
#[cfg(test)]
mod refuse_tests;
#[cfg(test)]
mod repeat_tests;
#[cfg(test)]
mod reveal_tests;
#[cfg(test)]
mod stage_tests;
#[cfg(test)]
mod stop_tests;
#[cfg(test)]
mod tests;

use std::time::Duration;

use super::guards::{answered, rejection, seen};
use super::note::{Note, Rejection};
use super::observe::{arrived, executed, refused, settled};
use super::proposal::{Proposal, hear};
use super::state::{ExitReason, Phase, State};
use super::{Facts, Outcome, Seen, Step, abort};
use crate::effect::{Effect, PageEffect, Target, Touch};
use crate::guard::identity::{Recheck, recheck};
use crate::judge::{Answers, JudgeError};

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
    fire: |state, _| Next::Out(Step::Finished(abort(state))),
}];

/// The rows that take the result of an effect.
///
/// Sleeping has no row of its own: waking up fits no row, and the mismatch
/// row takes it back to waiting, changing nothing else.
pub(super) static ROWS: [Action; 5] = [
    Action {
        enabled: |state, _| state.phase == Phase::Awaiting,
        fire: |state, now| on_reply(state, now, decide),
    },
    Action {
        enabled: |state, _| state.phase == Phase::Confirming,
        fire: |state, now| on_page(state, now, confirm),
    },
    Action {
        enabled: |state, _| state.phase == Phase::Acting,
        fire: |state, now| on_page(state, now, executed),
    },
    Action {
        enabled: |state, _| state.phase == Phase::Returning,
        fire: |state, now| on_page(state, now, arrived),
    },
    Action {
        enabled: |state, _| state.phase == Phase::Recovering,
        fire: |state, now| on_page(state, now, arrived),
    },
];

/// Takes the judge's reply with `fire`. Other facts do not fit the phase.
fn on_reply(
    state: State,
    now: &Now,
    fire: fn(State, &Result<Answers, JudgeError>, Duration) -> Next,
) -> Next {
    match answered(&now.facts) {
        Some(reply) => fire(state, reply, now.at),
        None => MISMATCH(state, now),
    }
}

/// Takes the observed page with `fire`. Other facts do not fit the phase.
fn on_page(state: State, now: &Now, fire: fn(State, Seen, Duration) -> Next) -> Next {
    match seen(&now.facts) {
        Some(seen) => fire(state, seen.clone(), now.at),
        None => MISMATCH(state, now),
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

/// Takes the judge's reply: a proposal to stop, finish with, or pass on.
fn decide(state: State, reply: &Result<Answers, JudgeError>, now: Duration) -> Next {
    match hear(reply, &state) {
        Ok(proposal) => judged(state, proposal, now),
        Err(anomaly) => idle(state.noted(Note::Anomaly(anomaly))),
    }
}

/// Stops the proposal, or else finishes with it or passes it on.
fn judged(state: State, proposal: Proposal, now: Duration) -> Next {
    match rejection(&state, &proposal) {
        Some(why) => idle(refused(state.noted(Note::Rejected(why)))),
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
fn confirm(state: State, seen: Seen, now: Duration) -> Next {
    // Confirming always holds the proposal: `carry_out` keeps it before
    // asking for the look-up. The phase is `Copy` and cannot carry it, so
    // the missing side stays, unreachable, treated as a vanished element.
    let checked = state
        .proposal
        .as_deref()
        .map_or(Err(Rejection::Missing), |p| verdict(p, &seen));
    match checked {
        Ok(touch) => touched(state, Phase::Acting, touch, now),
        Err(why) => idle(settled(refused(state.noted(Note::Rejected(why))), seen)),
    }
}

/// The operation, if the element is the same node as when the judge was asked.
fn verdict(proposal: &Proposal, seen: &Seen) -> Result<Touch, Rejection> {
    // A proposal without an identity never gets here: the guard
    // `unidentified` stops it while waiting for the judge.
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
