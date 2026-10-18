//! The decision budget and the time window.

use webqa_core::machine::State;

use super::view::{Property, Transition, asks};

/// The caps hold, and only questions and accesses use them up.
pub static LIMITS: [Property; 3] = [
    Property {
        name: "the budget and the window stay within their caps",
        holds: within_caps,
    },
    Property {
        name: "only a question uses one decision",
        holds: |t| t.after().budget().used() == t.before.budget().used() + u32::from(asks(t)),
    },
    Property {
        name: "only a browser operation counts in the window",
        holds: counts_accesses,
    },
];

fn within_caps(t: &Transition) -> bool {
    let after = t.after();
    after.budget().used() <= 3 && after.window().used(t.at) <= 2
}

fn counts_accesses(t: &Transition) -> bool {
    let used = |state: &State| state.window().used(t.at);
    used(t.after()) == used(t.before) + u32::from(t.touch().is_some())
}
