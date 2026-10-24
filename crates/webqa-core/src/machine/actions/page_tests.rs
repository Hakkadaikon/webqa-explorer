//! Observations after an effect: leaving a paywall, the counters after a
//! touch, and the first observation.

use std::collections::BTreeSet;

use crate::explore::{ElementOp, Memory, ScreenKey};
use crate::machine::testing::{
    answer, asking, away, below, counts, id, inside, pricing, rules, secs, seen, showed, started,
    told, top, touch, with,
};
use crate::machine::{ExitReason, Rules, State, Step, StuckKind, start};
use crate::screen::Screen;

/// Started on the paywall under `rules`: leaves with `op` without asking.
fn leaves(rules: Rules, op: &str) -> State {
    let step = started(rules, &pricing());
    assert_eq!(
        step.effect(),
        Some(&touch(op, None, 1.0)),
        "leaves with {op}"
    );
    assert_eq!(step.state().budget().used(), 0, "the judge is not asked");
    assert_eq!(step.state().window().used(secs(0)), 1, "one access");
    step.state().clone()
}

#[test]
fn paywall_leaves_without_asking() {
    let close = Rules {
        leave_paywall: "close".to_owned(),
        ..rules()
    };
    leaves(close, "close");
    let mut state = leaves(rules(), "back");
    for _ in 0..2 {
        let again = showed(&state, &pricing());
        let back = touch("back", None, 1.0);
        assert_eq!(again.effect(), Some(&back), "leaves again");
        state = again.state().clone();
    }
    let last = showed(&state, &pricing());
    let stuck = Some(ExitReason::Stuck(StuckKind::NoProgress));
    assert_eq!(last.reason(), stuck, "stuck after the third leave");
    assert_eq!(last.state().budget().used(), 0, "never asked");
}

/// Passes the answer `op` on `target`, then the pages until the touch is
/// observed on `then`, and returns the counts.
pub(super) fn after(
    state: &State,
    op: &str,
    target: Option<&str>,
    then: &Screen,
) -> (State, (u32, u32)) {
    let mut step: Step = told(state, answer(op, Some(0.9), target));
    if target.is_some() {
        step = showed(step.state(), &first());
    }
    let observed = showed(step.state(), then);
    let rules = rules();
    assert_eq!(
        observed.effect(),
        Some(&asking(&rules, then)),
        "{op} asks again"
    );
    let state = observed.state().clone();
    let counted = counts(&state);
    (state, counted)
}

/// The start page with "通知設定" below the viewport.
pub(super) fn first() -> Screen {
    with(top(), below("link", "通知設定"))
}

/// The same page scrolled so that "通知設定" is inside the viewport.
pub(super) fn scrolled() -> Screen {
    with(top(), inside("link", "通知設定"))
}

#[test]
fn touch_observation_updates_counters() {
    let state = started(rules(), &first()).state().clone();
    let (state, clicked) = after(&state, "click", Some("2"), &first());
    assert_eq!(clicked, (1, 0), "explored without a change");
    let (state, still) = after(&state, "scroll_down", None, &first());
    assert_eq!(still, (2, 1), "nothing new");
    let (state, revealed) = after(&state, "scroll_down", None, &scrolled());
    assert_eq!(revealed, (0, 0), "changed, and something new shows");
    let (state, up) = after(&state, "scroll_up", None, &first());
    assert_eq!(up, (0, 1), "changed, nothing new");
    let (_, again) = after(&state, "scroll_down", None, &scrolled());
    assert_eq!(again, (0, 2), "seen before");
}

/// Scrolls `role` `name` from below the viewport into it, starting with
/// `memory`, and returns the counts.
fn scrolling_in(role: &str, name: &str, memory: Memory) -> (u32, u32) {
    let hidden = with(top(), below(role, name));
    let asked = start(seen(&hidden), rules(), memory, secs(0));
    let shown = with(top(), inside(role, name));
    after(asked.state(), "scroll_down", None, &shown).1
}

#[test]
fn revealing_forbidden_or_operated_is_no_progress() {
    let forbidden = scrolling_in("button", "退会", Memory::default());
    assert_eq!(forbidden, (0, 1), "only a forbidden element shows");
    let mut memory = Memory::default();
    let at = memory.visit(&seen(&first()).screen, &BTreeSet::new());
    let notice = id("link", "通知設定", 0);
    memory.record(at, &notice, ElementOp::Click, at);
    let operated = scrolling_in("link", "通知設定", memory);
    assert_eq!(operated, (0, 1), "only an operated element shows");
}

#[test]
fn start_takes_first_observation() {
    let step = started(rules(), &top());
    assert_eq!(step.effect(), Some(&asking(&rules(), &top())), "asks");
    assert_eq!(
        step.state().memory().screen_count(),
        1,
        "remembers the page"
    );
    let outside = started(rules(), &away());
    assert_eq!(
        outside.effect(),
        Some(&touch("return", None, 1.0)),
        "returns"
    );
    assert!(outside.state().outside(), "outside");
    assert_eq!(outside.state().memory().screen_count(), 0, "not remembered");
    start_screen(step.state(), outside.state());
}

/// The start screen is the first page taken in inside the allowed origins.
fn start_screen(inside: &State, outside: &State) {
    let origin = Some(key_of(inside, &top()));
    assert_eq!(inside.origin(), origin, "the start page");
    assert_eq!(outside.origin(), None, "no start screen outside");
    let back = showed(outside, &first());
    let scrolled = told(back.state(), answer("scroll_down", Some(0.9), None));
    let later = showed(scrolled.state(), &top()).state().clone();
    let origin = Some(key_of(&later, &first()));
    assert_eq!(later.origin(), origin, "the first page inside starts");
    assert_ne!(origin, Some(key_of(&later, &top())), "not the later page");
}

/// The key of `screen` in the memory of `state`.
fn key_of(state: &State, screen: &Screen) -> ScreenKey {
    let mut memory = state.memory().clone();
    memory.visit(&seen(screen).screen, &BTreeSet::new())
}
