//! Answers that cannot become a proposal, and pages without elements.

use crate::config::raw::Op;
use crate::judge::{Answers, JudgeError};
use crate::machine::testing::{
    APP, answer, ask_top, asking, counts, heard, page, rules, secs, started, told, top, touch,
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

#[test]
fn unusable_answers_are_anomalies() {
    let failed = Err(JudgeError::Exhausted);
    let mut state = anomaly(
        &ask_top(rules()),
        failed,
        &Anomaly::Judge(JudgeError::Exhausted),
    );
    state = anomaly(
        &state,
        Ok(answer("drag", Some(0.9), None)),
        &Anomaly::Operation,
    );
    state = anomaly(&state, Ok(Answers::default()), &Anomaly::Operation);
    for number in ["99", "0", "x", "01"] {
        let click = Ok(answer("click", Some(0.9), Some(number)));
        state = anomaly(&state, click, &Anomaly::Target);
    }
    anomaly(
        &state,
        Ok(answer("click", Some(0.9), None)),
        &Anomaly::Target,
    );
    let without_wait = Rules {
        ops: vec![Op::Click, Op::Back],
        ..rules()
    };
    anomaly(
        &ask_top(without_wait),
        Ok(answer("wait", Some(0.9), None)),
        &Anomaly::Operation,
    );
}

#[test]
fn empty_screen_allows_untargeted_ops() {
    let empty = page(&format!("{APP}/"), &[]);
    let asked = started(rules(), &empty);
    assert_eq!(
        asked.effect(),
        Some(&asking(&rules(), &empty)),
        "no number to choose"
    );
    let scrolled = told(asked.state(), answer("scroll_down", Some(0.9), None));
    let scroll = touch("scroll_down", None, 0.9);
    assert_eq!(scrolled.effect(), Some(&scroll), "scrolls");
    let clicked = told(asked.state(), answer("click", Some(0.9), Some("1")));
    let note = Note::Anomaly(Anomaly::Target);
    assert_eq!(
        clicked.state().notes().last(),
        Some(&note),
        "nothing to click"
    );
}
