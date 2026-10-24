//! Why the run ended, and what the state shows of it.

use webqa_core::event::Mode;
use webqa_core::judge::QuestionId;
use webqa_core::machine::{ExitReason, Phase, StuckKind};

use super::candidates::{STOP, TWO};
use super::rows::not_an_option;
use super::view::{Property, Transition, all, implies};

const DECLARED: ExitReason = ExitReason::Stuck(StuckKind::Declared);
const NO_PROGRESS: ExitReason = ExitReason::Stuck(StuckKind::NoProgress);

/// Each reason is backed by the state or by the answer.
pub static REASONS: [Property; 9] = [
    BUDGET_BACKED,
    TIME_BACKED,
    Property {
        name: "stuck without progress only at the stuck threshold",
        holds: |t| {
            implies(
                t.reason() == Some(NO_PROGRESS),
                t.after().counters().is_stuck(TWO),
            )
        },
    },
    Property {
        name: "declared stuck only on the answer stuck",
        holds: |t| implies(t.reason() == Some(DECLARED), t.op == Some("stuck")),
    },
    Property {
        name: "the answer stuck offered that ends the run declares it stuck",
        holds: |t| {
            let narrowed = t.before.rules().mode == Mode::Explore;
            let refused = t.adds(&not_an_option(QuestionId::operation()));
            let offered = !all(&[narrowed, refused]);
            implies(
                all(&[t.reason().is_some(), t.op == Some("stuck"), offered]),
                t.reason() == Some(DECLARED),
            )
        },
    },
    Property {
        name: "reported done only on the answer done",
        holds: |t| {
            implies(
                t.reason() == Some(ExitReason::ReportedDone),
                t.op == Some("done"),
            )
        },
    },
    Property {
        name: "the last state has ended for the reason",
        holds: |t| {
            t.reason()
                .is_none_or(|r| t.after().phase() == Phase::Ended(r))
        },
    },
    Property {
        name: "an interruption does not end the run",
        holds: |t| t.reason() != Some(ExitReason::UserAborted),
    },
    Property {
        name: "the core never ends for a browser error",
        holds: |t| t.reason() != Some(ExitReason::Error),
    },
];

/// Without stages, the goal of a scenario is never met.
pub static NO_GOAL: [Property; 1] = [Property {
    name: "the goal does not end a scenario without stages",
    holds: |t| t.reason() != Some(ExitReason::GoalAchieved),
}];

/// The run going on is in a phase waiting for facts.
pub static GOING_ON: [Property; 1] = [Property {
    name: "a state to go on with has neither ended nor stayed idle",
    holds: |t| implies(t.step.effect().is_some(), going_on(t)),
}];

/// Out of budget and out of time at once ends for one of them.
pub static TIE: [Property; 3] = [
    BUDGET_BACKED,
    TIME_BACKED,
    Property {
        name: "out of budget and of time ends for one of them",
        holds: |t| {
            implies(
                tied(t),
                [ExitReason::BudgetExceeded, ExitReason::TimeExceeded]
                    .contains(&t.reason().unwrap_or(DECLARED)),
            )
        },
    },
];

const BUDGET_BACKED: Property = Property {
    name: "out of budget only when the budget is used up",
    holds: |t| {
        implies(
            t.reason() == Some(ExitReason::BudgetExceeded),
            t.after().budget().exhausted(),
        )
    },
};

const TIME_BACKED: Property = Property {
    name: "out of time only at the time limit",
    holds: |t| implies(t.reason() == Some(ExitReason::TimeExceeded), t.at >= STOP),
};

fn going_on(t: &Transition) -> bool {
    let phase = t.after().phase();
    all(&[phase != Phase::Idle, t.after().ended().is_none()])
}

/// Ended out of budget, at the time limit, neither on the goal nor stuck,
/// and not on an answer that ends the run by itself.
pub fn tied(t: &Transition) -> bool {
    let after = t.after();
    all(&[
        t.reason().is_some(),
        t.reason() != Some(ExitReason::GoalAchieved),
        after.budget().exhausted(),
        t.at >= STOP,
        !after.counters().is_stuck(TWO),
        !matches!(t.op, Some("stuck" | "done")),
    ])
}
