//! The question whether the goal is reached: asked with the goal of the
//! stage, and its answer not read.

use super::{asks_for, begun, login_with};
use crate::judge::{Question, QuestionId};
use crate::machine::testing::{
    ALL_OPS, EXPLORE_OPS_NO_INPUT, answer, ask_with, asked, asked_for, done_url, explore, rules,
    seen, showed, staged, told, top, touch,
};

/// Whether `questions` ask whether the goal is reached.
fn asks_goal(questions: &[Question]) -> bool {
    questions
        .iter()
        .any(|q| *q.id() == QuestionId::goal_reached())
}

#[test]
fn goal_question_follows_stage() {
    let with_goal = asked_for(&ALL_OPS, 2, Some("ログインする"));
    let without = asked(&rules(), 2);
    let exploring = asked_for(&EXPLORE_OPS_NO_INPUT, 2, None);
    let asks = [&with_goal, &without, &exploring].map(|q| asks_goal(q));
    assert_eq!(asks, [true, false, false], "only with a stage");
    let cases = [
        (staged(login_with(done_url("/home"))), with_goal),
        (rules(), without),
        (explore(), exploring),
    ];
    for (rules, questions) in cases {
        let step = begun(rules, seen(&top()));
        let expected = ask_with(&top(), questions);
        assert_eq!(step.effect(), Some(&expected), "the questions");
    }
}

#[test]
fn goal_reached_answer_does_not_finish() {
    let rules = staged(login_with(done_url("/home")));
    let asked = begun(rules, seen(&top()));
    let mut answers = answer("scroll_down", Some(0.9), None);
    let reached = vec![("true".to_owned(), 1.0)];
    answers.table.insert(QuestionId::goal_reached(), reached);
    let scroll = told(asked.state(), answers);
    let expected = touch("scroll_down", None, 0.9);
    assert_eq!(scroll.effect(), Some(&expected), "scrolls");
    let again = showed(scroll.state(), &top());
    assert_eq!(
        again.effect(),
        Some(&asks_for("ログインする")),
        "asks again"
    );
    let still = (again.state().stage(), again.reason());
    assert_eq!(still, (0, None), "still logging in");
}
