//! The counters through the state machine: steps without a browser operation.

use crate::judge::Answers;
use crate::machine::testing::{
    answer, asking, counts, inside, nz, rules, started, told, top, with,
};
use crate::machine::{Anomaly, ExitReason, Note, Rejection, Rules};

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
    four_times(&nowhere, &Note::Anomaly(Anomaly::Target));
}
