//! Waiting for the time window while the page needs fixing.

use super::{rules_with, sleep, waiting, wake};
use crate::effect::Effect;
use crate::explore::Memory;
use crate::machine::testing::{
    asking, away, counters, idle, next, nz, picked, pricing, secs, seen, top, touch, window,
};
use crate::machine::{Phase, State};
use crate::plan::Pick;

#[test]
fn window_full_waits_for_next_window() {
    let rules = rules_with(|r| r.requests_per_hour = nz(2));
    let state = waiting(rules.clone()).with_window(window(2, 2, secs(10)));
    assert_eq!(picked(&state, secs(20)), Pick::WaitWindow, "waits");
    let step = next(state, secs(20));
    assert_eq!(step.effect(), Some(&sleep(3600)), "until the next window");
    assert_eq!(step.state().phase(), Phase::Sleeping, "sleeping");
    assert_eq!(step.state().budget().used(), 0, "the judge is not asked");
    let woke = wake(step.state().clone(), secs(3600));
    assert_eq!(woke.effect(), Some(&asking(&rules, &top())), "asks then");
}

/// Waits at 20 seconds with the window used up, then wakes and does `fixed`.
fn deferred(state: State, fixed: &Effect) {
    let state = state.with_window(window(2, 2, secs(10)));
    assert_eq!(picked(&state, secs(20)), Pick::WaitWindow, "waits first");
    let step = next(state, secs(20));
    assert_eq!(step.effect(), Some(&sleep(3600)), "until the next window");
    let woke = wake(step.state().clone(), secs(3600));
    assert_eq!(woke.effect(), Some(fixed), "fixes the page then");
    assert_eq!(
        woke.state().window().used(secs(3600)),
        1,
        "in the new window"
    );
}

#[test]
fn window_full_defers_reload_return_and_leave() {
    let rules = rules_with(|r| r.requests_per_hour = nz(2));
    let frozen = waiting(rules.clone()).with_counters(counters(3, 0));
    deferred(frozen, &touch("reload", None, 1.0));
    let outside = idle(rules.clone(), seen(&away()), Memory::default());
    deferred(outside, &touch("return", None, 1.0));
    let paywall = idle(rules, seen(&pricing()), Memory::default());
    deferred(paywall, &touch("back", None, 1.0));
}
