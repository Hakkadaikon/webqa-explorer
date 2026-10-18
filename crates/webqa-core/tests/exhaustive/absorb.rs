//! An ended run stays ended, whatever is passed to it.

use std::time::Duration;

use webqa_core::machine::{Outcome, State, Step, abort};

use super::view::all;

/// Whether passing more facts to the ended `outcome` at `at`, which gave
/// `again`, ended the same way and changed nothing.
pub fn stays(outcome: &Outcome, again: &Step, at: Duration) -> bool {
    let (before, after) = (&outcome.state, again.state());
    let memory = |s: &State| (s.memory().screen_count(), s.memory().edge_count());
    all(&[
        again.reason() == Some(outcome.reason),
        after.budget() == before.budget(),
        after.window().used(at) == before.window().used(at),
        after.counters() == before.counters(),
        after.notes().len() == before.notes().len(),
        memory(after) == memory(before),
    ])
}

/// Whether interrupting the ended `outcome` keeps its reason.
pub fn aborts_alike(outcome: &Outcome) -> bool {
    abort((*outcome.state).clone()).reason == outcome.reason
}
