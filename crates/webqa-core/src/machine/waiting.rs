//! The row taken while waiting: what decides it, and what each pick does.

use std::time::Duration;

use super::actions::{Action, Next, finish, out, touched};
use super::counters::Counters;
use super::guards::exploring;
use super::offer::questions;
use super::proposal::Proposal;
use super::state::{ExitReason, Phase, State, StuckKind};
use crate::effect::{Effect, Outside, Touch, Wake};
use crate::event::Mode;
use crate::plan::{Delays, Ends, Goal, Pick, Trouble, Waiting, choose, explore_goal};
use crate::stage;
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

/// The first tier, on the latest observation: the `done_when` of the
/// current stage in a scenario, the exploration from the start screen in
/// explore mode.
fn goal(state: &State) -> Goal {
    match state.rules.mode {
        Mode::Scenario | Mode::Interview => {
            stage::goal(&state.rules.stages, state.stage, state.last.whole.page())
        }
        Mode::Explore => explore_goal(state.budget.used(), reachable(state)),
    }
}

/// Whether an untouched element is reachable from the start screen. With no
/// start screen yet, there may be, so the goal waits.
fn reachable(state: &State) -> bool {
    state
        .origin
        .is_none_or(|origin| state.memory.untouched_reachable(origin))
}

/// What a pick does.
struct Move {
    pick: Pick,
    fire: fn(State, Duration) -> Next,
}

/// What each pick of waiting does.
static MOVES: [Move; 10] = [
    Move {
        pick: Pick::Advance,
        fire: |state, _| advance(state),
    },
    Move {
        pick: Pick::Achieve,
        fire: |state, _| finish(state, ExitReason::GoalAchieved),
    },
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

/// Goes to the next stage and waits again, with the counts restarted. The
/// budget, the window and the memory go on.
fn advance(mut state: State) -> Next {
    state.stage = state.stage.saturating_add(1);
    state.counters = Counters::default();
    Next::Go(Box::new(state))
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
