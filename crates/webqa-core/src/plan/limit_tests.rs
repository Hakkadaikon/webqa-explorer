//! The rows taken while waiting, run through the state machine: the end of
//! the run, the time window, the time limit and the page in trouble.

mod outside;
mod time;
mod window;

use std::time::Duration;

use super::Pick;
use crate::effect::{Effect, Outside, Wake};
use crate::explore::Memory;
use crate::machine::testing::{
    asking, budget, counters, idle, next, nz, picked, rules, secs, seen, top, touch,
};
use crate::machine::{ExitReason, Facts, Heard, Phase, Rules, State, Step, StuckKind, step};

/// The background rules changed by `change`.
fn rules_with(change: impl FnOnce(&mut Rules)) -> Rules {
    let mut rules = rules();
    change(&mut rules);
    rules
}

/// Waiting on the start page under `rules`.
fn waiting(rules: Rules) -> State {
    idle(rules, seen(&top()), Memory::default())
}

/// A budget of 4 with `used` used.
fn spent(used: u32) -> State {
    let rules = rules_with(|r| r.max_decisions = nz(4));
    waiting(rules).with_budget(budget(4, used))
}

fn sleep(at: u64) -> Effect {
    Effect::Outside(Outside::Sleep(Wake { at: secs(at) }))
}

fn wake(state: State, at: Duration) -> Step {
    step(state, Facts::Outside(Heard::Woke), at)
}

#[test]
fn budget_exhausted_finishes() {
    let state = spent(4);
    assert_eq!(picked(&state, secs(0)), Pick::OutOfBudget, "out of budget");
    let step = next(state, secs(0));
    assert_eq!(step.reason(), Some(ExitReason::BudgetExceeded), "finished");
    assert_eq!(step.effect(), None, "nothing to do");
    assert_eq!(step.state().budget().used(), 4, "no decision used");
    assert_eq!(
        step.state().phase(),
        Phase::Ended(ExitReason::BudgetExceeded),
        "the last state is ended"
    );
}

#[test]
fn budget_one_left_still_asks() {
    let state = spent(3);
    assert_eq!(picked(&state, secs(0)), Pick::Ask, "one decision is left");
    let rules = state.rules().clone();
    let step = next(state, secs(0));
    assert_eq!(step.effect(), Some(&asking(&rules, &top())), "asks");
    assert_eq!(step.state().phase(), Phase::Awaiting, "waits for the judge");
    assert_eq!(step.state().budget().used(), 4, "the last decision is used");
    assert_eq!(step.reason(), None, "not finished");
}

#[test]
fn stuck_beats_budget() {
    let state = spent(4).with_counters(counters(0, 3));
    assert_eq!(picked(&state, secs(0)), Pick::Stuck, "stuck first");
    assert_eq!(
        next(state, secs(0)).reason(),
        Some(ExitReason::Stuck(StuckKind::NoProgress)),
        "stuck without progress"
    );
}

#[test]
fn budget_beats_freeze_recovery() {
    let state = spent(4).with_counters(counters(3, 0));
    assert_eq!(picked(&state, secs(0)), Pick::OutOfBudget, "no reload");
    let step = next(state, secs(0));
    assert_eq!(step.reason(), Some(ExitReason::BudgetExceeded), "finished");
    assert_eq!(step.effect(), None, "the page is not reopened");
    assert_eq!(step.state().window().used(secs(0)), 0, "no access");
}

#[test]
fn no_question_while_frozen() {
    let state = waiting(rules()).with_counters(counters(3, 0));
    assert_eq!(picked(&state, secs(0)), Pick::Reload, "reloads");
    let step = next(state, secs(0));
    assert_eq!(step.effect(), Some(&touch("reload", None, 1.0)), "reopens");
    let state = step.state();
    assert_eq!(state.phase(), Phase::Recovering, "recovering");
    assert_eq!(state.budget().used(), 0, "the judge is not asked");
    assert_eq!(state.counters().freeze(), 0, "the freeze count restarts");
    assert_eq!(state.window().used(secs(0)), 1, "one access");
}
