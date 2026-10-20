//! Tests of the issues of a trial that ran out of its decision budget.

use crate::interview::clue::Clue;
use crate::interview::issue::tests::fixture::{
    BUDGET_SPENT, confirm, count, record, run, seen, unsure,
};
use crate::interview::issue::tests::sample::{EXPECTED, hesitant_at_confirm};
use crate::interview::issue::{Shot, step_issues};

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
