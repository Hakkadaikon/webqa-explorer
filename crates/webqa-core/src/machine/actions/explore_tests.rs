//! Explore mode: no `done` to choose, and the candidates narrow the options.

mod goal;
mod moves;

use crate::effect::Effect;
use crate::judge::{Answers, QuestionId, Unusable};
use crate::machine::testing::{
    APP, EXPLORE_OPS, answer, ask_with, asked_for, asked_only, below, budget, explore, id, idle,
    look, memory_of, next, options_of, page, ranked, secs, seen, started, told, top, with,
};
use crate::machine::{Anomaly, Note, Rules, State, Step};
use crate::screen::{ElementId, Screen};

/// The start page with "通知設定" below the viewport.
fn top_tall() -> Screen {
    with(top(), below("link", "通知設定"))
}

/// The settings page with `elements`.
fn settings(elements: &[(&str, &str)]) -> Screen {
    page(&format!("{APP}/settings"), elements)
}

fn save() -> ElementId {
    id("button", "保存", 0)
}

fn help() -> ElementId {
    id("link", "ヘルプ", 0)
}

fn notice() -> ElementId {
    id("button", "通知", 0)
}

/// Every click remembered: "保存" to the settings, "ヘルプ" back to the
/// start page, and "通知" staying on the settings.
fn all_clicked() -> [(usize, ElementId, usize); 3] {
    [(0, save(), 1), (0, help(), 0), (1, notice(), 1)]
}

/// The anomaly of an option not asked of the question `id`.
fn not_an_option(id: QuestionId) -> Note {
    Note::Anomaly(Anomaly::Unusable(Unusable::NotAnOption(id)))
}

/// The options of the operation kind and of the element the effect asks.
fn options(effect: Option<&Effect>) -> [Vec<String>; 2] {
    [QuestionId::operation(), QuestionId::target()].map(|id| options_of(effect, &id))
}

#[test]
fn explore_questions_omit_done() {
    let asked = started(explore(), &top());
    let expected = ask_with(&top(), asked_for(&EXPLORE_OPS, 2, None));
    assert_eq!(asked.effect(), Some(&expected), "no done, no goal");
    let kinds = [
        "click",
        "input",
        "scroll_down",
        "scroll_up",
        "back",
        "wait",
        "stuck",
    ];
    let both = [
        kinds.map(str::to_owned).to_vec(),
        vec!["1".into(), "2".into()],
    ];
    assert_eq!(options(asked.effect()), both, "every kind but done");
    let goal = options_of(asked.effect(), &QuestionId::goal_reached());
    assert_eq!(goal, Vec::<String>::new(), "no goal question");
}

#[test]
fn explore_done_is_anomaly() {
    let asked = started(explore(), &top());
    let done = told(asked.state(), answer("done", Some(0.9), None));
    assert_eq!(done.reason(), None, "not reported done");
    let note = not_an_option(QuestionId::operation());
    assert_eq!(done.state().notes().last(), Some(&note), "an anomaly");
    assert_eq!(done.effect(), asked.effect(), "asks again");
    assert_eq!(done.state().budget().used(), 2, "asked twice");
}

/// Waiting on the tall start page under `rules` with the clicks `edges`
/// remembered, after one question.
fn waiting_with(rules: Rules, edges: &[(usize, ElementId, usize)]) -> Step {
    let screens = [&top_tall(), &settings(&[("button", "通知")])];
    let memory = memory_of(&screens, edges);
    let state = idle(rules, seen(&top_tall()), memory).with_budget(budget(20, 1));
    next(state, secs(0))
}

/// Each of `answers` to `state` records `note`.
fn refused(state: &State, answers: Vec<Answers>, note: &Note) {
    for answers in answers {
        let refused = told(state, answers);
        assert_eq!(refused.state().notes().last(), Some(note), "not offered");
    }
}

/// The first tier: only the untouched "ヘルプ".
fn untouched_tier() {
    let asked = waiting_with(explore(), &[(0, save(), 1)]);
    let expected = ask_with(&top_tall(), asked_only(&EXPLORE_OPS, 2, &[2]));
    assert_eq!(asked.effect(), Some(&expected), "only ヘルプ");
    let state = asked.state();
    let help_clicked = told(state, answer("click", Some(0.9), Some("2")));
    assert_eq!(help_clicked.effect(), Some(&look(help())), "ヘルプ");
    let clicks = vec![
        answer("click", Some(0.9), Some("1")),
        ranked("click", Some(0.9), &[("2", 0.8), ("1", 0.6)]),
    ];
    refused(state, clicks, &not_an_option(QuestionId::target()));
}

#[test]
fn explore_targets_follow_candidates() {
    untouched_tier();
    let unfinished = &all_clicked()[..2];
    let asked = waiting_with(explore(), unfinished);
    let expected = ask_with(&top_tall(), asked_only(&EXPLORE_OPS, 2, &[1]));
    assert_eq!(asked.effect(), Some(&expected), "only 保存");
    let clicked = told(asked.state(), answer("click", Some(0.9), Some("1")));
    assert_eq!(clicked.effect(), Some(&look(save())), "保存");
    moves::moves_tier();
}
