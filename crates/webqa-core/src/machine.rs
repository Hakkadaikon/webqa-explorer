//! The state machine of a run: a table of rows, each a condition and a
//! transition, following `docs/architecture.md`.
//!
//! The core is pure. The run loop does the effects it returns and passes
//! back what came of them with the time since the run started.

mod actions;
#[cfg(test)]
mod cause_tests;
pub mod counters;
mod guards;
mod material;
mod note;
mod observe;
mod offer;
mod ops;
mod proposal;
mod rules;
#[cfg(test)]
mod start_tests;
mod state;
#[cfg(test)]
pub(crate) mod testing;
#[cfg(test)]
mod tests;
mod waiting;
mod whole;

use std::time::Duration;

use crate::effect::Effect;
use crate::explore::Memory;
use crate::judge::{Answers, JudgeError};
use crate::redact::Redacted;
use crate::screen::{Screen, Viewport};
use crate::window::RequestWindow;
use actions::{Action, MISMATCH, Next, Now};

pub use material::Material;
pub use note::{Anomaly, Note, Rejection};
pub use rules::Rules;
pub use state::{Cause, ExitReason, Phase, State, StuckKind};
pub use whole::Whole;

/// A redacted observation of the page, with the material of the guardrails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    /// The page.
    pub screen: Redacted<Screen>,
    /// The size of the viewport.
    pub viewport: Viewport,
    /// What the guardrails judge by, collected before redaction.
    pub material: Material,
    /// The whole page that `done_when` reads.
    pub whole: Whole,
}

/// What the run loop found out by doing an effect.
#[derive(Debug, Clone, PartialEq)]
pub enum Facts {
    /// The result of an effect outside the page.
    Outside(Heard),
    /// The page after an effect on the page.
    Page(Box<Seen>),
}

/// The result of an effect outside the page.
#[derive(Debug, Clone, PartialEq)]
pub enum Heard {
    /// The judge model answered, or failed to.
    Answered(Result<Answers, JudgeError>),
    /// The wait is over.
    Woke,
}

/// How the run ended, and the state it ended in.
#[derive(Debug, Clone)]
pub struct Outcome {
    /// Why it ended.
    pub reason: ExitReason,
    /// The last state. Passing it on to [`step`] ends the same way again.
    pub state: Box<State>,
}

/// What the run loop gets back from the core.
#[derive(Debug, Clone)]
pub enum Step {
    /// Do the effect, then pass its facts with the state.
    Continue(Box<State>, Effect),
    /// The run is over.
    Finished(Outcome),
}

impl Step {
    /// The effect to do, if the run goes on.
    #[must_use]
    pub const fn effect(&self) -> Option<&Effect> {
        match self {
            Self::Continue(_, effect) => Some(effect),
            Self::Finished(_) => None,
        }
    }

    /// Why the run ended, if it has.
    #[must_use]
    pub const fn reason(&self) -> Option<ExitReason> {
        match self {
            Self::Continue(..) => None,
            Self::Finished(outcome) => Some(outcome.reason),
        }
    }

    /// The state to go on with, or the last one.
    #[must_use]
    pub fn state(&self) -> &State {
        match self {
            Self::Continue(state, _) => state,
            Self::Finished(outcome) => &outcome.state,
        }
    }
}

/// The table: the ended row first, then waiting, then the rest.
static TABLE: [&[Action]; 3] = [&actions::ENDED, &waiting::ROWS, &actions::ROWS];

/// Starts a run on the first observation, with the memory read from the store.
#[must_use]
pub fn start(seen: Seen, rules: Rules, memory: Memory, now: Duration) -> Step {
    let window = RequestWindow::new(rules.requests_per_hour);
    start_in(seen, rules, memory, window, now)
}

/// As [`start`], with the core's window `window` instead of a new one.
#[must_use]
pub fn start_in(
    seen: Seen,
    rules: Rules,
    memory: Memory,
    window: RequestWindow,
    now: Duration,
) -> Step {
    let state = State::observed(rules, seen, memory).with_window(window);
    let facts = Facts::Outside(Heard::Woke);
    settle(state, &Now { facts, at: now })
}

/// Takes the facts of the last effect at `now` and tells what to do next.
#[must_use]
pub fn step(state: State, facts: Facts, now: Duration) -> Step {
    settle(state, &Now { facts, at: now })
}

/// The user interrupted the run. An ended run keeps its reason.
#[must_use]
pub fn abort(state: State) -> Outcome {
    let reason = state.ended().unwrap_or(ExitReason::UserAborted);
    Outcome {
        reason,
        state: Box::new(state.to(Phase::Ended(reason))),
    }
}

/// Runs rows until one returns something for the run loop.
fn settle(state: State, now: &Now) -> Step {
    match fire(state, now) {
        Next::Go(state) => settle(*state, now),
        Next::Out(step) => step,
    }
}

/// Runs the first row that applies. Sleeping has no row: whatever comes
/// then goes back to waiting, as facts that do not fit the phase do.
fn fire(state: State, now: &Now) -> Next {
    let fire = TABLE
        .iter()
        .flat_map(|rows| rows.iter())
        .find(|row| (row.enabled)(&state, now))
        .map_or(MISMATCH, |row| row.fire);
    fire(state, now)
}
