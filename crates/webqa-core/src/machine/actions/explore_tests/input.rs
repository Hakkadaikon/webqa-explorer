//! Explore mode offers `input` only for an element that takes a value, and
//! counts only an `input` that typed.

use std::collections::BTreeSet;

use super::{APP, options, save};
use crate::config::raw::Op;
use crate::machine::testing::{answer, explore, id, operated, page, showed, started, told};
use crate::machine::{Rules, Step};
use crate::screen::Screen;

/// The start page: a text field "メール" and a button "保存".
pub(super) fn fielded() -> Screen {
    page(
        &format!("{APP}/"),
        &[("textbox", "メール"), ("button", "保存")],
    )
}

/// Explore mode with a value for each of `names`.
pub(super) fn valued(names: &[&str]) -> Rules {
    Rules {
        input_names: names
            .iter()
            .map(|n| (*n).to_owned())
            .collect::<BTreeSet<_>>(),
        ..explore()
    }
}

/// Whether the operation kinds offered at the start of `screen` have `input`.
fn offers_input(rules: Rules, screen: &Screen) -> bool {
    let asked = started(rules, screen);
    options(asked.effect())[0].contains(&"input".to_owned())
}

#[test]
fn input_is_offered_only_for_a_field_with_a_value() {
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
        .map(|(rules, screen, _)| offers_input(rules, &screen));
    assert_eq!(
        offered,
        cases.map(|(_, _, expected)| expected),
        "input offered"
    );
}

#[test]
fn input_stays_when_the_rules_offer_nothing_else() {
    let only = Rules {
        ops: vec![Op::Input],
        ..valued(&[])
    };
    let screen = page(&format!("{APP}/"), &[("button", "保存")]);
    assert!(offers_input(only, &screen), "the only kind stays");
}

/// Types into "メール", or into "保存", then looks at the page again.
fn typed(rules: Rules, number: &str) -> Step {
    let asked = started(rules, &fielded());
    let looked = told(asked.state(), answer("input", Some(0.9), Some(number)));
    let touched = showed(looked.state(), &fielded());
    showed(touched.state(), &fielded())
}

#[test]
fn input_is_remembered_only_when_it_typed() {
    let field = id("textbox", "メール", 0);
    let typing = typed(valued(&["メール"]), "1");
    assert!(operated(typing.state(), &fielded(), &field), "typed");
    let valueless = typed(valued(&[]), "1");
    assert!(!operated(valueless.state(), &fielded(), &field), "no value");
    let button = typed(valued(&["メール"]), "2");
    assert!(
        !operated(button.state(), &fielded(), &save()),
        "not a field"
    );
}
