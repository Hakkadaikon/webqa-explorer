//! The operation chosen is the one the core carries out.

use super::{Chosen, Pointed, chosen, stopped};
use crate::effect::Touch;
use crate::judge::{Answers, JudgeError};
use crate::machine::testing::{answer, ask_top, id, ranked, rules, showed, told, top, touch};
use crate::machine::{Rejection, State};

fn pointed(number: usize, name: &str) -> Pointed {
    Pointed {
        number,
        name: name.to_owned(),
    }
}

fn of(state: &State, answers: Answers) -> Option<Chosen> {
    chosen(&Ok(answers), state)
}

#[test]
fn an_operation_on_an_element_names_it() {
    let state = ask_top(rules());
    let click = ranked("click", Some(0.9), &[("2", 0.8), ("1", 0.6)]);
    let expected = Chosen {
        op: "click",
        element: Some(pointed(2, "ヘルプ")),
    };
    assert_eq!(of(&state, click), Some(expected), "the best number");
    let scroll = Chosen {
        op: "scroll_down",
        element: None,
    };
    let answers = answer("scroll_down", Some(0.9), None);
    assert_eq!(of(&state, answers), Some(scroll), "no element");
}

#[test]
fn nothing_is_chosen_when_the_core_carries_out_nothing() {
    let state = ask_top(rules());
    let cases = [
        ("done", answer("done", Some(0.9), None)),
        ("stuck", answer("stuck", Some(0.9), None)),
        ("doubtful back", answer("back", Some(0.5), None)),
        ("not offered", answer("drag", Some(0.9), None)),
    ];
    for (name, answers) in cases {
        assert_eq!(of(&state, answers), None, "{name}");
    }
    let failed = chosen(&Err(JudgeError::Exhausted), &state);
    assert_eq!(failed, None, "no answers");
}

#[test]
fn the_element_passed_over_for_being_unmoved_is_the_one_named() {
    let ranks = || ranked("click", Some(0.9), &[("1", 0.8), ("2", 0.6)]);
    let looked = told(&ask_top(rules()), ranks());
    let touched = showed(looked.state(), &top());
    let expected = touch("click", Some(id("button", "保存", 0)), 0.9);
    assert_eq!(touched.effect(), Some(&expected), "clicks the first");
    let unchanged = showed(touched.state(), &top());
    let next = Chosen {
        op: "click",
        element: Some(pointed(2, "ヘルプ")),
    };
    assert_eq!(of(unchanged.state(), ranks()), Some(next), "the next one");
}

#[test]
fn stopped_names_the_operation_and_why() {
    let state = ask_top(rules());
    let back = Ok(answer("back", Some(0.5), None));
    let back_touch = Touch {
        op: "back".to_owned(),
        target: None,
        confidence: 0.5,
    };
    let why = Some((back_touch, Rejection::LowConfidence));
    assert_eq!(stopped(&back, &state), why, "a doubtful back");
    let scroll = Ok(answer("scroll_down", Some(0.9), None));
    assert_eq!(stopped(&scroll, &state), None, "carried out");
    let failed = stopped(&Err(JudgeError::Exhausted), &state);
    assert_eq!(failed, None, "no answers");
}
