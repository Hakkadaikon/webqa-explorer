//! The element of an `input`: the highest ranked one that takes a value.

use std::collections::BTreeSet;

use crate::effect::Effect;
use crate::machine::Rules;
use crate::machine::testing::{APP, id, look, page, ranked, rules, started, told};
use crate::screen::Screen;

/// A text field "メール" and a button "保存".
fn fielded() -> Screen {
    page(
        &format!("{APP}/"),
        &[("textbox", "メール"), ("button", "保存")],
    )
}

/// What the answer `op` with "保存" ranked over "メール" looks up, with a
/// value for each of `names`.
fn looked_up(op: &str, names: &[&str]) -> Option<Effect> {
    let input_names: BTreeSet<String> = names.iter().map(|n| (*n).to_owned()).collect();
    let asked = started(
        Rules {
            input_names,
            ..rules()
        },
        &fielded(),
    );
    let answers = ranked(op, Some(0.9), &[("2", 0.8), ("1", 0.6)]);
    told(asked.state(), answers).effect().cloned()
}

#[test]
fn input_goes_to_the_highest_ranked_element_that_takes_a_value() {
    let field = look(id("textbox", "メール", 0));
    let button = look(id("button", "保存", 0));
    let got = [
        looked_up("input", &["メール"]),
        looked_up("input", &[]),
        looked_up("click", &["メール"]),
    ];
    let want = [Some(field), Some(button.clone()), Some(button)];
    assert_eq!(got, want, "a field with a value, else the judge's pick");
}
