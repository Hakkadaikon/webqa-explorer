//! The time limit, alone and with the decision budget and the window.

use super::{rules_with, sleep, waiting, wake};
use crate::event::Mode;
use crate::machine::testing::{asking, budget, next, nz, picked, rules, secs, top, window};
use crate::machine::{ExitReason, Rules, State};
use crate::plan::Pick;

/// Rules of `mode` with a time limit of `limit` seconds.
fn limited(mode: Mode, limit: u64) -> Rules {
    rules_with(|r| {
        r.mode = mode;
        r.max_duration = Some(secs(limit));
    })
}

#[test]
fn time_limit_finishes_without_asking() {
    let state = waiting(limited(Mode::Explore, 60));
    assert_eq!(picked(&state, secs(60)), Pick::TimeUp, "time is up");
    let step = next(state, secs(60));
    assert_eq!(step.reason(), Some(ExitReason::TimeExceeded), "finished");
    assert_eq!(step.state().budget().used(), 0, "the judge is not asked");
}

/// At `at`, `rules` picks `pick` and gives `reason`, or asks when `None`.
fn limit_at(rules: &Rules, at: u64, pick: Pick, reason: Option<ExitReason>) {
    let state = waiting(rules.clone());
    assert_eq!(picked(&state, secs(at)), pick, "the row at {at}s");
    let step = next(state, secs(at));
    assert_eq!(step.reason(), reason, "the end at {at}s");
    let asks = reason.is_none().then(|| asking(rules, &top()));
    assert_eq!(step.effect(), asks.as_ref(), "the effect at {at}s");
}

#[test]
fn time_limit_boundary() {
    let limit = limited(Mode::Scenario, 60);
    limit_at(&limit, 59, Pick::Ask, None);
    limit_at(&limit, 60, Pick::TimeUp, Some(ExitReason::TimeExceeded));
    limit_at(&limit, 61, Pick::TimeUp, Some(ExitReason::TimeExceeded));
    limit_at(&rules(), 1_000_000, Pick::Ask, None);
}

#[test]
fn time_limit_applies_to_scenario_mode() {
    let rules = limited(Mode::Scenario, 300);
    limit_at(&rules, 300, Pick::TimeUp, Some(ExitReason::TimeExceeded));
    limit_at(&rules, 299, Pick::Ask, None);
}

#[test]
fn budget_and_timeout_yield_one_of_them() {
    let rules = rules_with(|r| {
        r.max_decisions = nz(4);
        r.max_duration = Some(secs(60));
    });
    let state = waiting(rules).with_budget(budget(4, 4));
    let pick = picked(&state, secs(60));
    assert!(
        [Pick::OutOfBudget, Pick::TimeUp].contains(&pick),
        "one of the two ends"
    );
    let step = next(state, secs(60));
    let reasons = [ExitReason::BudgetExceeded, ExitReason::TimeExceeded];
    let reason = step.reason().unwrap();
    assert!(reasons.contains(&reason), "one of the two reasons");
    assert_eq!(step.state().budget().used(), 4, "no decision used");
}

/// Waiting at 10 seconds with a full window of 2 and a time limit of `limit` seconds.
fn full_until(limit: u64) -> State {
    let rules = rules_with(|r| {
        r.requests_per_hour = nz(2);
        r.max_duration = Some(secs(limit));
    });
    waiting(rules).with_window(window(2, 2, secs(10)))
}

#[test]
fn a_window_opening_at_the_limit_or_later_ends_the_run_at_once() {
    for limit in [60, 3600] {
        let step = next(full_until(limit), secs(10));
        assert_eq!(step.reason(), Some(ExitReason::TimeExceeded), "{limit}s");
        assert_eq!(step.effect(), None, "no wait for {limit}s");
    }
}

#[test]
fn a_window_opening_before_the_limit_is_waited_for() {
    let state = full_until(3601);
    assert_eq!(picked(&state, secs(10)), Pick::WaitWindow, "waits");
    let step = next(state, secs(10));
    assert_eq!(step.effect(), Some(&sleep(3600)), "until the window opens");
    let woke = wake(step.state().clone(), secs(3600));
    assert_eq!(woke.reason(), None, "goes on");
}
