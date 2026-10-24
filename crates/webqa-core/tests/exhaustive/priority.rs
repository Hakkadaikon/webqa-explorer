//! The first tier of waiting comes first: the goal of the mode, the stages
//! going forward, and the goal achieved only when it holds.

use webqa_core::effect::{Effect, Outside, PageEffect, Touch};
use webqa_core::event::Mode;
use webqa_core::machine::counters::Counters;
use webqa_core::machine::{ExitReason, State, Step, StuckKind};
use webqa_core::plan::{Goal, explore_goal};
use webqa_core::stage;

use super::rows::{Row, row};
use super::view::{Property, Transition, all, asks, implies};

/// The goal of the mode on `state`, with `used` decisions used.
pub fn goal_of(state: &State, used: u32) -> Goal {
    let rules = state.rules();
    match rules.mode {
        Mode::Scenario | Mode::Interview => {
            stage::goal(&rules.stages, state.stage(), state.last().whole.page())
        }
        Mode::Explore => explore_goal(used, reachable(state)),
    }
}

/// Whether an untouched element is reachable from the start screen. With
/// no start screen yet, there may be.
fn reachable(state: &State) -> bool {
    state
        .origin()
        .is_none_or(|origin| state.memory().untouched_reachable(origin))
}

/// The reasons of the second and third tiers.
const LATER_TIERS: [ExitReason; 4] = [
    ExitReason::Stuck(StuckKind::NoProgress),
    ExitReason::JudgeUnavailable,
    ExitReason::BudgetExceeded,
    ExitReason::TimeExceeded,
];

/// An operation of the core's own: sure and on no element.
fn own(touch: &Touch) -> bool {
    touch.target.is_none() && (touch.confidence - 1.0).abs() < f64::EPSILON
}

/// The step came from a row of waiting below the first tier.
fn waited(step: &Step) -> bool {
    step.effect().map_or_else(|| ended_later(step), waiting)
}

/// The effect of a row of waiting: asking, sleeping, or an operation of
/// the core's own.
fn waiting(effect: &Effect) -> bool {
    match effect {
        Effect::Outside(_) => true,
        Effect::Page(PageEffect::Touch(touch)) => own(touch),
        Effect::Page(PageEffect::Look(_)) => false,
    }
}

/// The run ended in the second or third tier.
fn ended_later(step: &Step) -> bool {
    step.reason().is_some_and(|r| LATER_TIERS.contains(&r))
}

/// The decisions used before the step: a question has used one.
fn used_before(step: &Step) -> u32 {
    let used = step.state().budget().used();
    used.saturating_sub(u32::from(asked(step)))
}

const fn asked(step: &Step) -> bool {
    matches!(step.effect(), Some(Effect::Outside(Outside::Ask(_))))
}

/// A row below the first tier is taken only when the goal is unmet.
pub fn goal_comes_first(step: &Step) -> bool {
    implies(
        waited(step),
        goal_of(step.state(), used_before(step)) == Goal::Unmet,
    )
}

/// The goal is achieved only when it holds.
pub fn achieved_backed(step: &Step) -> bool {
    let state = step.state();
    implies(
        step.reason() == Some(ExitReason::GoalAchieved),
        goal_of(state, state.budget().used()) == Goal::Achieved,
    )
}

/// The stage goes forward from `before`, past met stages only, within the
/// stages, with the counts restarted. Explore mode stays in the first.
pub fn stages_go_forward(before: usize, step: &Step) -> bool {
    let state = step.state();
    let stages = &state.rules().stages;
    let now = state.stage();
    let page = state.last().whole.page();
    let passed = (before..now).all(|k| stage::goal(stages, k, page) == Goal::Advance);
    let restarted = implies(now > before, state.counters() == Counters::default());
    let exploring = state.rules().mode == Mode::Explore;
    all(&[
        now >= before,
        now < stages.len().max(1),
        passed,
        restarted,
        implies(exploring, now == 0),
    ])
}

/// Whether the first step of a run keeps the priority.
pub fn start_holds(step: &Step) -> bool {
    all(&[
        goal_comes_first(step),
        achieved_backed(step),
        stages_go_forward(0, step),
    ])
}

/// The priority of the first tier on every transition.
pub static PRIORITY: [Property; 3] = [
    Property {
        name: "a row below the first tier only when the goal is unmet",
        holds: |t| goal_comes_first(t.step),
    },
    Property {
        name: "the goal is achieved only when it holds",
        holds: |t| achieved_backed(t.step),
    },
    Property {
        name: "the stage goes forward past met stages, with the counts restarted",
        holds: |t| stages_go_forward(t.before.stage(), t.step),
    },
];

/// Whether the transition went to a later stage.
fn advanced(t: &Transition) -> bool {
    t.after().stage() > t.before.stage()
}

/// What the search with stages must reach.
pub fn staged_rows() -> Vec<Row> {
    vec![
        row("go to a stage, then ask", |t| all(&[advanced(t), asks(t)])),
        row("go through the stages to the goal", |t| {
            all(&[advanced(t), t.reason() == Some(ExitReason::GoalAchieved)])
        }),
        row("go to a stage outside, then return", |t| {
            all(&[advanced(t), t.touches("return")])
        }),
    ]
}
