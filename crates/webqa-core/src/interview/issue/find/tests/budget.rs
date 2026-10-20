//! Tests of the issues of a trial that ran out of its decision budget.

use crate::interview::clue::Clue;
use crate::interview::issue::tests::fixture::{
    BUDGET_SPENT, NO_PROGRESS, confirm, count, record, run, seen, shipping, unsure,
};
use crate::interview::issue::tests::sample::{
    EXPECTED, hesitant_at_confirm, unreached_at_shipping,
};
use crate::interview::issue::{Halt, Issue, Shot, step_issues};

#[test]
fn budget_spent_trial_keeps_hesitation_without_unreached() {
    let (confirm, unsure) = (confirm(), unsure());
    let steps = [seen(&confirm, Some(&unsure), &[Clue::LowConfidence], None)];
    let got = step_issues(&run(&[record(BUDGET_SPENT, &steps)]));
    let hesitated = hesitant_at_confirm(
        &[Clue::LowConfidence],
        EXPECTED,
        count(1, 1, 1),
        Shot { trial: 1, step: 1 },
        vec![],
    );
    assert_eq!(got, vec![hesitated]);
}

#[test]
fn budget_spent_trials_stay_in_n_of_the_unreached_issue() {
    let shipping = shipping();
    let steps = [seen(&shipping, None, &[], None)];
    let stuck = record(NO_PROGRESS, &steps);
    let spent = record(BUDGET_SPENT, &steps);
    let got = step_issues(&run(&[stuck, spent, spent]));
    let expected = Issue {
        count: count(1, 3, 3),
        shot: Some(Shot { trial: 1, step: 1 }),
        path: vec![],
        ..unreached_at_shipping(Halt::NoProgress, None)
    };
    assert_eq!(got, vec![expected]);
}
