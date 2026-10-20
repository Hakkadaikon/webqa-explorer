//! `enter` presses Enter where the focus is: it is offered when a field
//! shown takes a value, never with a forbidden element on the page, and it
//! asks for no element.

use super::input::{fielded, valued};
use super::{APP, notice, options, save, settings};
use crate::config::raw::Op;
use crate::machine::testing::{
    answer, below, budget, id, idle, inside, memory_of, next, operated, page, rules, secs, seen,
    showed, started, told, touch, with,
};
use crate::machine::{Rules, State, Step};
use crate::screen::Screen;

/// Whether the operation kinds offered at the start of `screen` have `enter`.
fn offers_enter(rules: Rules, screen: &Screen) -> bool {
    let asked = started(rules, screen);
    options(asked.effect())[0].contains(&"enter".to_owned())
}

#[test]
fn enter_is_offered_only_with_a_field_that_takes_a_value() {
    let links = page(
        &format!("{APP}/"),
        &[("link", "メール"), ("button", "保存")],
    );
    let cases = [
        (valued(&["メール"]), fielded(), true),
        (valued(&[]), fielded(), false),
        (valued(&["保存"]), fielded(), false),
        (valued(&["メール"]), links, false),
    ];
    let offered = cases
        .clone()
        .map(|(rules, screen, _)| offers_enter(rules, &screen));
    assert_eq!(
        offered,
        cases.map(|(_, _, expected)| expected),
        "enter offered"
    );
}

/// Types into the first element "メール" of `screen`, and looks at it again.
fn typed(screen: &Screen) -> Step {
    let asked = started(valued(&["メール"]), screen);
    let looked = told(asked.state(), answer("input", Some(0.9), Some("1")));
    let touched = showed(looked.state(), screen);
    showed(touched.state(), screen)
}

/// The operation kinds offered when the field and the other elements shown
/// are all operated, so that only the moves are left.
fn moves_options() -> Vec<String> {
    let start = with(fielded(), below("link", "通知設定"));
    let field = id("textbox", "メール", 0);
    let edges = [(0, field, 1), (0, save(), 1), (1, notice(), 1)];
    let screens = [&start, &settings(&[("button", "通知")])];
    let state = idle(
        valued(&["メール"]),
        seen(&start),
        memory_of(&screens, &edges),
    );
    let moves = next(state.with_budget(budget(20, 1)), secs(0));
    options(moves.effect())[0].clone()
}

#[test]
fn enter_stays_after_the_field_is_typed() {
    let screen = fielded();
    let asked = typed(&screen);
    let field = id("textbox", "メール", 0);
    assert!(operated(asked.state(), &screen, &field), "operated");
    let untouched = options(asked.effect())[0].clone();
    assert!(untouched.contains(&"enter".to_owned()), "{untouched:?}");
    let moves = moves_options();
    let kinds = ["scroll_down", "scroll_up", "back", "enter"].map(str::to_owned);
    assert_eq!(moves, kinds.to_vec(), "the moves and enter");
}

#[test]
fn enter_is_dropped_when_a_forbidden_element_is_on_the_page() {
    let risky = with(fielded(), inside("button", "購入する"));
    let cases = [
        (valued(&["メール"]), fielded(), true),
        (valued(&["メール"]), risky.clone(), false),
        (rules(), fielded(), true),
        (rules(), risky, false),
    ];
    let offered = cases
        .clone()
        .map(|(rules, screen, _)| offers_enter(rules, &screen));
    assert_eq!(
        offered,
        cases.map(|(_, _, expected)| expected),
        "enter offered"
    );
}

#[test]
fn enter_needs_no_target() {
    let screen = fielded();
    let state = typed(&screen).state().clone();
    let counts = |state: &State| {
        let ids = [id("textbox", "メール", 0), save()];
        (
            state.memory().screen_count(),
            ids.map(|e| operated(state, &screen, &e)),
        )
    };
    let pressed = told(&state, answer("enter", Some(0.9), None));
    assert_eq!(
        pressed.effect(),
        Some(&touch("enter", None, 0.9)),
        "pressed with no element"
    );
    let after = showed(pressed.state(), &screen);
    assert_eq!(counts(after.state()), counts(&state), "the memory is kept");
}

#[test]
fn enter_stays_when_the_rules_offer_nothing_else() {
    let risky = with(fielded(), inside("button", "購入する"));
    let only = Rules {
        ops: vec![Op::Enter],
        ..valued(&[])
    };
    let asked = started(only, &risky);
    assert_eq!(options(asked.effect())[0], ["enter"], "no empty question");
}
