//! The row taken while waiting: what decides it, and what each pick does.

use std::time::Duration;

use super::actions::{Action, Next, finish, out, questions, touched};
use super::guards::exploring;
use super::proposal::Proposal;
use super::state::{ExitReason, Phase, State, StuckKind};
use crate::effect::{Effect, Outside, Touch, Wake};
use crate::plan::{Delays, Ends, Goal, Pick, Trouble, Waiting, choose};
use crate::window::wake_at;

/// The waiting row of the table.
pub(super) static ROWS: [Action; 1] = [Action {
    enabled: |state, _| exploring(state),
    fire: |state, now| wait(state, now.at),
}];

/// What decides the row taken while waiting at `now`.
pub(super) fn waiting(state: &State, now: Duration) -> Waiting {
    Waiting {
        goal: goal(state),
        ends: Ends {
            stuck: state.counters.is_stuck(state.rules.stuck_threshold),
            out_of_budget: state.budget.exhausted(),
            time_up: state.rules.max_duration.is_some_and(|limit| now >= limit),
        },
        delays: Delays {
            window_full: state.window.full(now),
            trouble: Trouble {
                frozen: state.counters.frozen(state.rules.freeze_threshold),
                outside: state.outside,
                paywall: state.rules.paywall.detects(&state.last.material.paywall),
            },
        },
    }
}

/// The first tier. Never met until the stages and the goal of explore mode
/// are wired.
const fn goal(_: &State) -> Goal {
    Goal::Unmet
}

/// What a pick does.
struct Move {
    pick: Pick,
    fire: fn(State, Duration) -> Next,
}

/// The picks this state machine takes. Going to the next stage and
/// achieving the goal come with the first tier.
static MOVES: [Move; 8] = [
    Move {
        pick: Pick::Stuck,
        fire: |state, _| finish(state, ExitReason::Stuck(StuckKind::NoProgress)),
    },
    Move {
        pick: Pick::OutOfBudget,
        fire: |state, _| finish(state, ExitReason::BudgetExceeded),
    },
    Move {
        pick: Pick::TimeUp,
        fire: |state, _| finish(state, ExitReason::TimeExceeded),
    },
    Move {
        pick: Pick::WaitWindow,
        fire: sleep,
    },
    Move {
        pick: Pick::Reload,
        fire: reload,
    },
    Move {
        pick: Pick::Return,
        fire: |state, now| own(state, Phase::Returning, "return", now),
    },
    Move {
        pick: Pick::Leave,
        fire: leave,
    },
    Move {
        pick: Pick::Ask,
        fire: ask,
    },
];

/// What a pick does when it has no move of its own.
const ASK: fn(State, Duration) -> Next = ask;

/// Takes the row that the priority of waiting picks at `now`.
fn wait(state: State, now: Duration) -> Next {
    let pick = choose(&waiting(&state, now));
    let fire = MOVES
        .iter()
        .find(|m| m.pick == pick)
        .map_or(ASK, |m| m.fire);
    fire(state, now)
}

/// Sleeps until the next window, or until the time limit if earlier.
fn sleep(state: State, now: Duration) -> Next {
    let at = wake_at(state.window.next_start(now), state.rules.max_duration);
    out(
        state,
        Phase::Sleeping,
        Effect::Outside(Outside::Sleep(Wake { at })),
    )
}

/// Reopens the page; the freeze count restarts.
fn reload(mut state: State, now: Duration) -> Next {
    state.counters = state.counters.after_reload();
    own(state, Phase::Recovering, "reload", now)
}

/// Leaves the paywall the way the rules say, as an operation to observe.
fn leave(mut state: State, now: Duration) -> Next {
    let op = state.rules.leave_paywall.clone();
    state.proposal = Some(Box::new(Proposal::leave(op.clone())));
    own(state, Phase::Acting, &op, now)
}

/// An operation of the core's own, sure and on no element.
fn own(state: State, phase: Phase, op: &str, now: Duration) -> Next {
    let touch = Touch {
        op: op.to_owned(),
        target: None,
        confidence: 1.0,
    };
    touched(state, phase, touch, now)
}

/// Asks the judge, using one decision.
fn ask(mut state: State, _: Duration) -> Next {
    state.budget = state.budget.spend();
    let questions = questions(&state);
    out(
        state,
        Phase::Awaiting,
        Effect::Outside(Outside::Ask(questions)),
    )
}
