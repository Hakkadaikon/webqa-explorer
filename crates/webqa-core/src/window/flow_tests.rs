//! The time window counted through the state machine.

use crate::effect::{Effect, Outside, Wake};
use crate::explore::Memory;
use crate::machine::testing::{
    away, counters, idle, next, nz, pricing, rules, secs, seen, shows, top, touch, window,
};
use crate::machine::{Rules, State, step};

/// `state` with one access used at 10 seconds of 2, does `fixed` at 20 seconds,
/// then waits for the next window instead of asking.
fn counted(state: State, fixed: &Effect) {
    let state = state.with_window(window(2, 1, secs(10)));
    let fixing = next(state, secs(20));
    assert_eq!(fixing.effect(), Some(fixed), "fixes the page");
    assert_eq!(fixing.state().window().used(secs(20)), 2, "counted");
    let after = step(fixing.state().clone(), shows(&top()), secs(20));
    let sleep = Effect::Outside(Outside::Sleep(Wake { at: secs(3600) }));
    assert_eq!(after.effect(), Some(&sleep), "waits for the next window");
}

#[test]
fn counts_each_access_kind() {
    let rules = Rules {
        requests_per_hour: nz(2),
        ..rules()
    };
    let frozen = idle(rules.clone(), seen(&top()), Memory::default()).with_counters(counters(3, 0));
    counted(frozen, &touch("reload", None, 1.0));
    let outside = idle(rules.clone(), seen(&away()), Memory::default());
    counted(outside, &touch("return", None, 1.0));
    let paywall = idle(rules, seen(&pricing()), Memory::default());
    counted(paywall, &touch("back", None, 1.0));
}
