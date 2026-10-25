//! The element of an `input`: the highest ranked one that takes a value.

use std::collections::BTreeSet;

use crate::effect::Effect;
use crate::judge::Answers;
use crate::machine::testing::{APP, id, look, page, ranked, rules, showed, started, told};
use crate::machine::{Rules, State};
use crate::screen::{ElementId, Screen};

/// A button "保存" and the text fields "メール" and "備考".
fn fielded() -> Screen {
    page(
        &format!("{APP}/"),
        &[
            ("button", "保存"),
            ("textbox", "メール"),
            ("textbox", "備考"),
        ],
    )
}

fn save() -> ElementId {
    id("button", "保存", 0)
}

fn mail() -> ElementId {
    id("textbox", "メール", 0)
}

fn note() -> ElementId {
    id("textbox", "備考", 0)
}

/// Asking about `fielded` with a value for each of `names`.
fn asking(names: &[&str]) -> State {
    let input_names: BTreeSet<String> = names.iter().map(|n| (*n).to_owned()).collect();
    let rules = Rules {
        input_names,
        ..rules()
    };
    started(rules, &fielded()).state().clone()
}

/// The answer `op` ranking the elements numbered `order`.
fn ranking(op: &str, order: &[&str]) -> Answers {
    let pairs: Vec<(&str, f64)> = order.iter().copied().zip([0.8, 0.6, 0.4]).collect();
    ranked(op, Some(0.9), &pairs)
}

/// What the answer looks up next.
fn looked_up(state: &State, answers: Answers) -> Option<Effect> {
    told(state, answers).effect().cloned()
}

#[test]
fn input_goes_to_the_highest_ranked_element_that_takes_a_value() {
    let cases = [
        (&["メール", "備考"][..], "input", note()),
        (&["保存", "メール"], "input", mail()),
        (&[], "input", save()),
        (&["メール"], "click", save()),
    ];
    let (got, want): (Vec<_>, Vec<_>) = cases
        .map(|(names, op, element)| {
            let got = looked_up(&asking(names), ranking(op, &["1", "3", "2"]));
            (got, Some(look(element)))
        })
        .into_iter()
        .unzip();
    assert_eq!(got, want, "a field with a value, else the judge's pick");
}

/// Types into the field `element` the answer points at, and the screen
/// stays the same.
fn typed_unchanged(state: &State, answers: Answers, element: ElementId) -> State {
    let looked = told(state, answers);
    assert_eq!(looked.effect(), Some(&look(element)), "looks the field up");
    let touched = showed(looked.state(), &fielded());
    showed(touched.state(), &fielded()).state().clone()
}

#[test]
fn input_passes_over_a_field_left_unmoved_only_for_another_field() {
    let order = ["1", "2", "3"];
    let two = typed_unchanged(
        &asking(&["メール", "備考"]),
        ranking("input", &order),
        mail(),
    );
    let next = looked_up(&two, ranking("input", &order));
    assert_eq!(next, Some(look(note())), "the other field");
    let one = typed_unchanged(&asking(&["メール"]), ranking("input", &order), mail());
    let again = looked_up(&one, ranking("input", &order));
    assert_eq!(again, Some(look(mail())), "the one field, not the button");
}
