//! Tests of gathering, ranking, naming and showing the issues.

mod counting;
pub mod fixture;
pub mod sample;

use fixture::{
    CLICK1, CUT, GOAL, NO_PROGRESS, answered, cart, clicked, confirm, count, guessing, next1,
    product, record, run, seen, sure, unsure,
};
use sample::{
    EXPECTED, hesitant_at_confirm, mismatch_sample, sample, unreached_at_cart, unreached_sample,
    visibility_sample,
};

use super::*;
use crate::interview::ask::Visit;

#[test]
fn cut_trial_clues_are_dropped() {
    let (cart, confirm, sure, unsure) = (cart(), confirm(), sure(), unsure());
    let first = [answered(&cart, &sure)];
    let second = [
        first[0],
        seen(&confirm, Some(&unsure), &[Clue::Hesitant], None),
    ];
    let trials = [record(NO_PROGRESS, &first), record(CUT, &second)];
    let got = step_issues(&run(&trials));
    assert_eq!(got, vec![unreached_at_cart()], "the finished trial only");
    let cut = trials.map(|trial| record(CUT, trial.steps));
    assert_eq!(step_issues(&run(&cut)), vec![], "every trial cut");
    assert_eq!(step_issues(&run(&[])), vec![], "no trial");
}

/// Three trials: cart then confirm, with the first and third trials
/// taking `steps` steps, and only the second trial hesitating at confirm.
fn hesitant_once(steps: usize) -> Vec<Issue> {
    let (cart, confirm, sure, unsure) = (cart(), confirm(), sure(), unsure());
    let walked = [
        seen(&cart, Some(&sure), &[], CLICK1),
        answered(&confirm, &sure),
    ];
    let second = [
        walked[0],
        seen(&confirm, Some(&unsure), &[Clue::Hesitant], None),
    ];
    let other = record(GOAL, &walked[..steps]);
    step_issues(&run(&[other, record(GOAL, &second), other]))
}

/// The hesitation at confirm in the second of three trials, with
/// `reached` trials reaching confirm.
fn hesitated(reached: usize) -> Vec<Issue> {
    let shot = Shot { trial: 2, step: 2 };
    let clues = [Clue::Hesitant];
    let issue = hesitant_at_confirm(&clues, EXPECTED, count(1, 3, reached), shot, clicked());
    vec![issue]
}

#[test]
fn single_occurrence_kept_with_count() {
    assert_eq!(hesitant_once(2), hesitated(3), "all reach confirm");
    assert_eq!(hesitant_once(1), hesitated(1), "one reaches confirm");
}

/// An issue to sort: `(number, k, N)` of a sample, or `None` for the
/// mismatch sample.
type Ranked = Option<(&'static str, usize, usize)>;

/// Issues to sort, and their numbers once sorted.
const RANKED: [(&[Ranked], &[&str]); 5] = [
    (
        &[
            Some(("I-00000001", 1, 3)),
            Some(("I-00000002", 3, 3)),
            Some(("I-00000003", 2, 3)),
        ],
        &["I-00000002", "I-00000003", "I-00000001"],
    ),
    (
        &[Some(("I-00000002", 2, 3)), Some(("I-00000001", 2, 3))],
        &["I-00000001", "I-00000002"],
    ),
    (
        &[Some(("I-00000001", 2, 4)), Some(("I-00000002", 1, 2))],
        &["I-00000001", "I-00000002"],
    ),
    (
        &[Some(("I-00000001", 2, 3)), None],
        &["I-D0D06221", "I-00000001"],
    ),
    (
        &[None, Some(("I-00000001", 3, 3))],
        &["I-00000001", "I-D0D06221"],
    ),
];

fn ranked(written: Ranked) -> Issue {
    written.map_or_else(mismatch_sample, |(number, k, n)| sample(number, k, n))
}

#[test]
fn rank_orders_by_rate_then_number() {
    for (i, (written, expected)) in RANKED.into_iter().enumerate() {
        let mut issues: Vec<Issue> = written.iter().copied().map(ranked).collect();
        issues.sort_by(rank);
        let got: Vec<String> = issues.iter().map(|it| it.number.to_string()).collect();
        assert_eq!(got, expected, "case {i}");
    }
}

/// The rows of `table` whose label is not the name written in it.
fn mislabelled<T: Copy>(table: &[(T, &str)], label: fn(T) -> &'static str) -> usize {
    table
        .iter()
        .filter(|&&(row, name)| label(row) != name)
        .count()
}

#[test]
fn samples_have_their_kinds() {
    let samples = [
        sample("I-5334A6B3", 1, 3),
        visibility_sample(),
        unreached_sample(),
        mismatch_sample(),
    ];
    let names = ["判断に迷う", "視認性が悪い", "目的に届かない", "目的のずれ"];
    assert_eq!(samples.map(|it| it.kind().label()), names, "of each sample");
    let wrong = (
        mislabelled(&KINDS, Kind::label),
        mislabelled(&CLUES, clue_label),
        mislabelled(&HALTS, Halt::label),
    );
    assert_eq!(wrong, (0, 0, 0), "the labels of the tables");
}

/// Page strings and judge text that `Debug` of an issue leaves out.
const HIDDEN: [&str; 5] = [
    "次へ",
    "クーポンを使う",
    "shop.example.test",
    "押すと",
    "会員登録",
];

#[test]
fn debug_hides_page_strings() {
    let samples = [
        (sample("I-5334A6B3", 1, 3), "I-5334A6B3", "Hesitant"),
        (visibility_sample(), "I-39E24428", "Visibility"),
        (unreached_sample(), "I-47871971", "Unreached"),
        (mismatch_sample(), "I-D0D06221", "Mismatch"),
    ];
    for (issue, number, kind) in samples {
        let shown = format!("{issue:?}");
        let marks = (shown.contains(number), shown.contains(kind));
        assert_eq!(marks, (true, true), "{shown}");
        let leaked: Vec<&str> = HIDDEN.into_iter().filter(|s| shown.contains(s)).collect();
        assert_eq!(leaked, Vec::<&str>::new(), "{number}");
    }
    let target = format!("{:?}", next1());
    let marks = (target.contains("次へ"), target.contains("button"));
    assert_eq!(marks, (false, true), "{target}");
}

#[test]
fn visits_follow_steps() {
    let (product, cart) = (product(42), cart());
    let guessed = guessing("会員登録をする");
    let steps = [answered(&product, &guessed), seen(&cart, None, &[], None)];
    let expected = vec![
        Visit {
            url: &product.as_inner().url,
            guess: Some("会員登録をする"),
        },
        Visit {
            url: &cart.as_inner().url,
            guess: None,
        },
    ];
    assert_eq!(visits(&record(GOAL, &steps)), expected, "one per step");
    assert_eq!(visits(&record(GOAL, &[])), vec![], "no step");
}
