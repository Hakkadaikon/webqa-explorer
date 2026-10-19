//! Tests of counting the trials of an issue and of the ledger order.

use super::fixture::{GOAL, answered, cart, confirm, count, record, run, seen, sure, unsure};
use super::sample::{EXPECTED, hesitant_at_confirm, sample};
use crate::interview::clue::Clue;
use crate::interview::issue::{Issue, Shot, rank, step_issues};

#[test]
fn issue_twice_in_one_trial_counts_that_trial_once() {
    let (cart, confirm, sure, unsure) = (cart(), confirm(), sure(), unsure());
    let first = [
        answered(&cart, &sure),
        seen(&confirm, Some(&unsure), &[Clue::Hesitant], None),
        seen(&confirm, Some(&unsure), &[Clue::Hesitant], None),
    ];
    let second = [answered(&cart, &sure)];
    let got = step_issues(&run(&[record(GOAL, &first), record(GOAL, &second)]));
    let shot = Shot { trial: 1, step: 2 };
    let expected = hesitant_at_confirm(&[Clue::Hesitant], EXPECTED, count(1, 2, 1), shot, vec![]);
    assert_eq!(got, vec![expected], "one trial of two");
}

/// The samples of every `k` of `n` trials with `n` up to four, numbered
/// from 1 in the order of `rates`.
fn rates() -> Vec<(usize, usize)> {
    (1..=4).flat_map(|n| (1..=n).map(move |k| (k, n))).collect()
}

fn rate((k, n): (usize, usize)) -> f64 {
    f64::from(u8::try_from(k).unwrap()) / f64::from(u8::try_from(n).unwrap())
}

fn issue_of(number: usize, (k, n): (usize, usize)) -> Issue {
    sample(&format!("I-{number:08X}"), k, n)
}

#[test]
fn rank_compares_rates_of_any_two_counts() {
    for (i, &left) in rates().iter().enumerate() {
        for (j, &right) in rates().iter().enumerate() {
            let by_rate = rate(right).partial_cmp(&rate(left)).unwrap();
            let expected = by_rate.then(i.cmp(&j));
            let got = rank(&issue_of(i + 1, left), &issue_of(j + 1, right));
            assert_eq!(got, expected, "{left:?} against {right:?}");
        }
    }
}
