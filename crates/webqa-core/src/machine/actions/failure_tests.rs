//! The judge failing several times in a row ends the run.

use crate::judge::{Answers, JudgeError};
use crate::machine::testing::{answer, ask_top, asking, heard, nz, rules, showed, told, top};
use crate::machine::{ExitReason, Rules, State, Step};

/// The background rules with `decisions` decisions.
fn budgeted(decisions: u32) -> Rules {
    Rules {
        max_decisions: nz(decisions),
        ..rules()
    }
}

/// Passes each of `replies` in turn from waiting for the judge on the start
/// page, keeping the failure count and the end after each.
fn replied(
    state: State,
    replies: Vec<Result<Answers, JudgeError>>,
) -> Vec<(u32, Option<ExitReason>)> {
    let mut state = state;
    let mut seen = Vec::new();
    for reply in replies {
        let step = heard(&state, reply);
        seen.push((step.state().failing(), step.reason()));
        state = step.state().clone();
    }
    seen
}

/// The state after `n` API failures from waiting on the start page.
fn failed(state: State, n: usize) -> State {
    (0..n).fold(state, |s, _| {
        heard(&s, Err(JudgeError::Api)).state().clone()
    })
}

#[test]
fn three_judge_failures_in_a_row_end_the_run() {
    let failures = vec![
        Err(JudgeError::Api),
        Err(JudgeError::Timeout),
        Err(JudgeError::Bridge),
    ];
    let state = ask_top(budgeted(20));
    let ended = Some(ExitReason::JudgeUnavailable);
    let seen = replied(state.clone(), failures);
    assert_eq!(seen, vec![(1, None), (2, None), (3, ended)]);
    let second = heard(&failed(state.clone(), 1), Err(JudgeError::Api));
    let asks = asking(&budgeted(20), &top());
    assert_eq!(second.effect(), Some(&asks), "asks again after two");
    assert_eq!(failed(state, 3).budget().used(), 3, "three decisions used");
}

/// A usable answer on the start page, then the start page again.
fn answered(state: &State) -> Step {
    let scrolled = told(state, answer("scroll_down", Some(0.9), None));
    showed(scrolled.state(), &top())
}

#[test]
fn an_answer_restarts_the_failure_count() {
    let state = ask_top(budgeted(20));
    let twice = replied(state.clone(), vec![Err(JudgeError::Api); 2]);
    assert_eq!(twice, vec![(1, None), (2, None)]);
    let again = answered(&failed(state, 2));
    assert_eq!(again.state().failing(), 0, "restarted");
    let seen = replied(again.state().clone(), vec![Err(JudgeError::Api); 2]);
    assert_eq!(seen, vec![(1, None), (2, None)], "goes on");
}

#[test]
fn unusable_answers_are_not_judge_failures() {
    let unusable = [
        JudgeError::Malformed,
        JudgeError::Exhausted,
        JudgeError::Unreadable {
            question: "expectation".to_owned(),
        },
    ];
    for error in unusable {
        let seen = replied(ask_top(budgeted(20)), vec![Err(error.clone()); 5]);
        assert_eq!(seen, vec![(0, None); 5], "{error:?}");
    }
    let state = ask_top(budgeted(20));
    let mixed = vec![
        Err(JudgeError::Api),
        Err(JudgeError::Api),
        Err(JudgeError::Malformed),
        Err(JudgeError::Api),
    ];
    assert_eq!(
        replied(state, mixed),
        vec![(1, None), (2, None), (0, None), (1, None)]
    );
}

#[test]
fn judge_failures_come_before_the_budget() {
    let seen = replied(ask_top(budgeted(3)), vec![Err(JudgeError::Api); 3]);
    let ended = Some(ExitReason::JudgeUnavailable);
    assert_eq!(seen, vec![(1, None), (2, None), (3, ended)]);
    let short = replied(ask_top(budgeted(2)), vec![Err(JudgeError::Api); 2]);
    let spent = Some(ExitReason::BudgetExceeded);
    assert_eq!(
        short,
        vec![(1, None), (2, spent)],
        "two failures are not enough"
    );
}
