//! Answers that cannot become a proposal, and pages without elements.

use std::iter;

use crate::config::raw::Op;
use crate::judge::{Answers, JudgeError, QuestionId, Unusable};
use crate::machine::testing::{
    APP, answer, ask_top, ask_with, asked, asking, counts, heard, page, rules, secs, started, told,
    top, touch,
};
use crate::machine::{Anomaly, Note, Rules, State};

/// Passes `answers` and checks the anomaly and that the judge is asked again.
fn anomaly(state: &State, answers: Result<Answers, JudgeError>, expected: &Anomaly) -> State {
    let used = state.budget().used();
    let step = heard(state, answers);
    let after = step.state();
    let note = Note::Anomaly(expected.clone());
    assert_eq!(
        after.notes().last(),
        Some(&note),
        "{expected:?} is recorded"
    );
    let rules = after.rules().clone();
    assert_eq!(
        step.effect(),
        Some(&asking(&rules, &top())),
        "{expected:?} asks again"
    );
    assert_eq!(
        after.budget().used(),
        used + 1,
        "{expected:?} asks once more"
    );
    assert_eq!(counts(after), (0, 0), "{expected:?} leaves the counts");
    assert_eq!(
        after.window().used(secs(0)),
        0,
        "{expected:?} uses no access"
    );
    after.clone()
}

/// The answers that cannot be read, because `id` was `problem`.
fn unusable(problem: fn(QuestionId) -> Unusable, id: QuestionId) -> Anomaly {
    Anomaly::Unusable(problem(id))
}

/// Answers with an operation kind or an element that was not offered.
fn not_offered() -> Vec<(Answers, Anomaly)> {
    let operation = unusable(Unusable::NotAnOption, QuestionId::operation());
    let target = unusable(Unusable::NotAnOption, QuestionId::target());
    let numbers = ["99", "0", "x", "01"];
    let clicks = numbers.map(|n| (answer("click", Some(0.9), Some(n)), target.clone()));
    let drag = (answer("drag", Some(0.9), None), operation);
    iter::once(drag).chain(clicks).collect()
}

/// Answers without an answer to the operation kind, the element, or
/// whether the page shows an error.
fn missing() -> Vec<(Answers, Anomaly)> {
    let mut no_error = answer("scroll_down", Some(0.9), None);
    no_error.table.remove(&QuestionId::error_shown());
    let cases = [
        (Answers::default(), QuestionId::operation()),
        (answer("click", Some(0.9), None), QuestionId::target()),
        (no_error, QuestionId::error_shown()),
    ];
    let missing = |(answers, id)| (answers, unusable(Unusable::Missing, id));
    cases.into_iter().map(missing).collect()
}

#[test]
fn unusable_answers_are_anomalies() {
    let failed = Err(JudgeError::Exhausted);
    let judge = Anomaly::Judge(JudgeError::Exhausted);
    let mut state = anomaly(&ask_top(rules()), failed, &judge);
    for (answers, expected) in not_offered().into_iter().chain(missing()) {
        state = anomaly(&state, Ok(answers), &expected);
    }
    let without_wait = Rules {
        ops: vec![Op::Click, Op::Back],
        ..rules()
    };
    let wait = Ok(answer("wait", Some(0.9), None));
    let operation = unusable(Unusable::NotAnOption, QuestionId::operation());
    anomaly(&ask_top(without_wait), wait, &operation);
}

#[test]
fn empty_screen_allows_untargeted_ops() {
    let empty = page(&format!("{APP}/"), &[]);
    let asked_empty = started(rules(), &empty);
    let questions = ask_with(&empty, asked(&rules(), 0));
    assert_eq!(
        asked_empty.effect(),
        Some(&questions),
        "no number to choose"
    );
    let scrolled = told(asked_empty.state(), answer("scroll_down", Some(0.9), None));
    let scroll = touch("scroll_down", None, 0.9);
    assert_eq!(scrolled.effect(), Some(&scroll), "scrolls");
    let clicked = told(asked_empty.state(), answer("click", Some(0.9), Some("1")));
    let target = unusable(Unusable::NotAnOption, QuestionId::target());
    let note = Note::Anomaly(target);
    assert_eq!(
        clicked.state().notes().last(),
        Some(&note),
        "nothing to click"
    );
}
