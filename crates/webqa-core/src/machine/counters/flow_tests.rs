//! The counters through the state machine: steps without a browser
//! operation, and going to the next stage.

use crate::config::raw::Stage;
use crate::explore::Memory;
use crate::judge::{Answers, QuestionId, Unusable};
use crate::machine::testing::{
    ALL_OPS, answer, ask_with, asked_for, asking, counters, counts, done_url, idle, inside, next,
    nz, rules, secs, seen, showed, stage, staged, started, told, top, two, with,
};
use crate::machine::{Anomaly, ExitReason, Note, Rejection, Rules, State, Step, StuckKind};

/// Answers the same `answers` four times on a page with "退会".
fn four_times(answers: &Answers, note: &Note) {
    let rules = Rules {
        max_decisions: nz(4),
        ..rules()
    };
    let screen = with(top(), inside("button", "退会"));
    let asks = asking(&rules, &screen);
    let mut step = started(rules, &screen);
    for _ in 0..4 {
        assert_eq!(step.effect(), Some(&asks), "{note:?}: only questions");
        step = told(step.state(), answers.clone());
        assert_eq!(counts(step.state()), (0, 0), "{note:?}: counts kept");
    }
    assert_eq!(
        step.reason(),
        Some(ExitReason::BudgetExceeded),
        "{note:?}: ends"
    );
    let notes = vec![note.clone(); 4];
    assert_eq!(step.state().notes(), notes.as_slice(), "{note:?}: recorded");
}

#[test]
fn steps_without_browser_op_leave_counters() {
    let forbidden = answer("click", Some(0.9), Some("3"));
    four_times(&forbidden, &Note::Rejected(Rejection::Forbidden));
    let unsure = answer("back", Some(0.5), None);
    four_times(&unsure, &Note::Rejected(Rejection::LowConfidence));
    let nowhere = answer("click", Some(0.9), Some("99"));
    let target = Anomaly::Unusable(Unusable::NotAnOption(QuestionId::target()));
    four_times(&nowhere, &Note::Anomaly(target));
}

/// Scrolls down on the start page, which does not change.
fn scrolled(state: &State) -> Step {
    let touched = told(state, answer("scroll_down", Some(0.9), None));
    showed(touched.state(), &top())
}

/// Waiting on the start page under `stages`, two steps from frozen and stuck.
fn nearly_stuck(stages: Vec<Stage>) -> Step {
    let waiting = idle(staged(stages), seen(&top()), Memory::default());
    next(waiting.with_counters(counters(2, 2)), secs(10))
}

#[test]
fn stage_advance_resets_freeze_and_stuck() {
    let advanced = nearly_stuck(two());
    let asks = ask_with(&top(), asked_for(&ALL_OPS, 2, Some("投稿する")));
    assert_eq!(advanced.effect(), Some(&asks), "asks about posting");
    let mut state = advanced.state().clone();
    assert_eq!((state.stage(), counts(&state)), (1, (0, 0)), "restarted");
    for expected in [(1, 1), (2, 2)] {
        let observed = scrolled(&state);
        assert_eq!(observed.effect(), Some(&asks), "{expected:?}: asks again");
        state = observed.state().clone();
        assert_eq!(counts(&state), expected, "counted from zero");
    }
    goes_on_without_a_stage();
}

/// The counts go on to stuck when no stage is done.
fn goes_on_without_a_stage() {
    let unmet = vec![
        stage("ログイン", "ログインする", done_url("/home")),
        two().remove(1),
    ];
    let last = scrolled(nearly_stuck(unmet).state());
    assert_eq!(counts(last.state()), (3, 3), "the counts go on");
    let no_progress = Some(ExitReason::Stuck(StuckKind::NoProgress));
    assert_eq!(last.reason(), no_progress, "stuck without a new stage");
}
