//! The exploration memory, the notes and the counters.

use webqa_core::machine::Phase;

use super::view::{Property, Transition, all, any, implies};

/// What was learned is kept, and the counters move only by steps.
pub static MEMORY: [Property; 4] = [
    Property {
        name: "an operated element stays operated",
        holds: operated_stays,
    },
    Property {
        name: "the notes only grow",
        holds: |t| t.after().notes().starts_with(t.before.notes()),
    },
    Property {
        name: "reloading restarts the freeze count",
        holds: |t| implies(t.touches("reload"), t.after().counters().freeze() == 0),
    },
    Property {
        name: "the counters move only by observing an operation, landing outside, reloading, or going to a stage",
        holds: |t| {
            let advanced = t.after().stage() > t.before.stage();
            any(&[
                t.before.phase() == Phase::Acting,
                advanced,
                counts(t) == expected(t),
            ])
        },
    },
];

fn operated_stays(t: &Transition) -> bool {
    let before = t.before_marks.iter().flatten();
    let after = t.after_marks.iter().flatten();
    before.zip(after).all(|(was, is)| implies(*was, *is))
}

fn counts(t: &Transition) -> (u32, u32) {
    let counters = t.after().counters();
    (counters.freeze(), counters.stuck())
}

/// The counts before, with one more step without progress on landing
/// outside without an operation, and the freeze count restarted on reloading.
fn expected(t: &Transition) -> (u32, u32) {
    let counters = t.before.counters();
    let freeze = counters.freeze() * u32::from(!t.touches("reload"));
    (freeze, counters.stuck() + u32::from(stalled(t)))
}

/// Landed outside on a page taken in without counting an operation.
fn stalled(t: &Transition) -> bool {
    let settling = [Phase::Confirming, Phase::Returning, Phase::Recovering];
    all(&[settling.contains(&t.before.phase()), t.after().outside()])
}
