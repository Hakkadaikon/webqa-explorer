//! The third tier of explore mode: going back and scrolling only, unless
//! the rules have none of them.

use super::{all_clicked, not_an_option, options, refused, top_tall, waiting_with};
use crate::config::raw::Op;
use crate::effect::{Effect, Outside};
use crate::judge::QuestionId;
use crate::machine::Rules;
use crate::machine::testing::{
    APP, MOVES, answer, ask_with, asked_for, asked_only, explore, page, started, told, touch,
};

/// The third tier on the tall start page: only the moves.
pub(super) fn moves_tier() {
    let asked = waiting_with(explore(), &all_clicked());
    let expected = ask_with(&top_tall(), asked_only(&MOVES, 2, &[]));
    assert_eq!(asked.effect(), Some(&expected), "only the moves");
    let moves = ["scroll_down", "scroll_up", "back"].map(str::to_owned);
    assert_eq!(options(asked.effect()), [moves.to_vec(), vec![]], "moves");
    let state = asked.state();
    let clicks = vec![answer("click", Some(0.9), Some("1"))];
    refused(state, clicks, &not_an_option(QuestionId::operation()));
    let back = told(state, answer("back", Some(0.9), None));
    assert_eq!(back.effect(), Some(&touch("back", None, 0.9)), "back");
    let unasked: [Vec<String>; 2] = Default::default();
    assert_eq!(options(back.effect()), unasked, "a touch asks nothing");
}

#[test]
fn explore_moves_offer_back_and_scrolls() {
    let empty = page(&format!("{APP}/empty"), &[]);
    let asked = started(explore(), &empty);
    let expected = ask_with(&empty, asked_only(&MOVES, 0, &[]));
    assert_eq!(asked.effect(), Some(&expected), "only the moves");
    let scrolled = told(asked.state(), answer("scroll_down", Some(0.9), None));
    let expected = touch("scroll_down", None, 0.9);
    assert_eq!(scrolled.effect(), Some(&expected), "scrolls");
    let clicks = Rules {
        ops: vec![Op::Click],
        ..explore()
    };
    let unnarrowed = waiting_with(clicks, &all_clicked());
    let expected = ask_with(&top_tall(), asked_for(&[Op::Click], 2, None));
    assert_eq!(unnarrowed.effect(), Some(&expected), "clicks still");
}

#[test]
fn explore_moves_ask_no_element() {
    let asked = waiting_with(explore(), &all_clicked());
    let Some(Effect::Outside(Outside::Ask(sent))) = asked.effect() else {
        panic!("asks");
    };
    let ids: Vec<&str> = sent.questions.iter().map(|q| q.id().as_str()).collect();
    assert_eq!(ids, ["operation", "error_shown"], "no element to choose");
}
