//! Tests of the cap on the questions asked after the trials.

pub mod fixture;

use std::num::NonZeroU32;

use fixture::{ids_of, mismatch_of, numbers, patterns, quota, shows, three_purposes, unnatural};

use super::*;
use crate::interview::issue::tests::fixture::products;
use crate::interview::issue::tests::sample::{mismatch_sample, sample, unreached_sample};
use crate::interview::issue::{Issue, UncheckedPurpose};

#[test]
fn asks_once_per_issue_outside_budget() {
    let mut cap = quota(50);
    let issues = vec![
        sample("I-00000001", 2, 3),
        sample("I-00000002", 1, 3),
        unreached_sample(),
    ];
    let (asked, over) = natural_round(&mut cap, issues);
    let expected = ["I-00000001", "I-00000002", "I-47871971"];
    assert_eq!(numbers(&asked), expected, "by rate, then number");
    assert_eq!(over, vec![], "none over the cap");
    assert_eq!(cap.left(), 47, "one question per issue");
    let mut once = numbers(&asked);
    once.dedup();
    assert_eq!(once.len(), 3, "each number once");
    for issue in &asked {
        let both = ["natural", "persona_trait"].map(str::to_owned).to_vec();
        assert_eq!(ids_of(issue), (both, vec![]), "{:?}", issue.number);
    }
}

/// The issues in the order passed: ranked `I-00000003`, `I-00000001`,
/// `I-00000002`.
fn unranked() -> Vec<Issue> {
    vec![
        sample("I-00000002", 1, 3),
        sample("I-00000003", 3, 3),
        sample("I-00000001", 2, 3),
    ]
}

/// The cap, the number of issues passed (the first of [`unranked`]), and
/// the numbers asked, over the cap, and left.
const CAPPED: [(u32, usize, usize, usize, usize); 4] = [
    (2, 3, 2, 1, 0),
    (3, 3, 3, 0, 0),
    (50, 3, 3, 0, 47),
    (1, 0, 0, 0, 1),
];

#[test]
fn cap_asks_in_ledger_order_rest_unchecked() {
    let mut cap = quota(2);
    let (asked, over) = natural_round(&mut cap, unranked());
    assert_eq!(numbers(&asked), ["I-00000003", "I-00000001"], "ranked");
    let unchecked = Placed {
        issue: sample("I-00000002", 1, 3),
        section: Section::Unchecked,
        persona_trait: None,
    };
    assert_eq!(over, vec![unchecked], "the last one is unchecked");
    assert_eq!(cap.left(), 0, "used up");
    for (n, passed, ask, rest, left) in CAPPED {
        let mut cap = quota(n);
        let issues = unranked().into_iter().take(passed).collect();
        let (asked, over) = natural_round(&mut cap, issues);
        let got = (asked.len(), over.len(), cap.left());
        assert_eq!(got, (ask, rest, left), "cap {n} with {passed} issues");
    }
}

/// The numbers of mismatch questions asked and over the cap, and what is
/// left, after asking `count` of the three under a cap of `n`.
fn mismatch_counts(n: u32, count: usize) -> (usize, usize, usize) {
    let screens = three_purposes();
    let mut cap = quota(n);
    let questions = mismatch_of(&screens).into_iter().take(count).collect();
    let (asked, over) = mismatch_round(&mut cap, questions);
    (asked.len(), over.len(), cap.left())
}

#[test]
fn mismatch_questions_take_cap_before_natural() {
    let screens = three_purposes();
    let mut cap = quota(2);
    let (asked, over) = mismatch_round(&mut cap, mismatch_of(&screens));
    assert_eq!(patterns(&asked), ["/products/*", "/cart"], "as written");
    let shipping = UncheckedPurpose {
        pattern: "/shipping".to_owned(),
        purpose: "配送先を入力する".to_owned(),
    };
    assert_eq!(over, vec![shipping], "over the cap");
    assert_eq!(cap.left(), 0, "used up by the mismatch questions");
    let (asked, over) = natural_round(&mut cap, vec![sample("I-00000001", 2, 3)]);
    assert_eq!(asked, vec![], "no naturalness question");
    let sections: Vec<Section> = over.iter().map(|it| it.section).collect();
    assert_eq!(sections, [Section::Unchecked], "the issue is unchecked");
    assert_eq!(mismatch_counts(50, 3), (3, 0, 47), "all three under 50");
    assert_eq!(mismatch_counts(50, 0), (0, 0, 50), "no mismatch question");
}

#[test]
fn mismatch_issue_also_gets_natural_question() {
    let screens = [products()];
    let mut cap = quota(50);
    let (asked, _) = mismatch_round(&mut cap, mismatch_of(&screens));
    assert_eq!(patterns(&asked), ["/products/*"], "one mismatch question");
    assert_eq!(cap.left(), 49, "after the mismatch question");
    let issues = vec![sample("I-00000001", 2, 3), mismatch_sample()];
    let (asked, over) = natural_round(&mut cap, issues);
    let expected = vec![mismatch_sample(), sample("I-00000001", 2, 3)];
    assert_eq!(asked, expected, "rate 1 first");
    assert_eq!(over, vec![], "none over the cap");
    assert_eq!(cap.left(), 47, "one question per issue");
    let lines = [
        "書いた目的: 商品をカートに入れる",
        "推測: 会員登録をする、商品をカートに入れる",
    ];
    assert_eq!(shows(&mismatch_sample(), lines), [true; 2], "{lines:?}");
    let placed = place(mismatch_sample(), Some(unnatural()));
    assert_eq!(placed.section, Section::OffPersona, "not natural");
}

#[test]
fn quota_takes_only_what_is_left() {
    let mut cap = quota(2);
    assert_eq!(cap.take(vec![1, 2, 3]), (vec![1, 2], vec![3]), "up to 2");
    assert_eq!(cap.left(), 0, "used up");
    assert_eq!(cap.take(vec![4]), (vec![], vec![4]), "nothing left");
    let mut cap = quota(5);
    assert_eq!(cap.take(Vec::<u8>::new()), (vec![], vec![]), "empty");
    assert_eq!(cap.left(), 5, "nothing taken");
    let whole = Quota::new(NonZeroU32::MAX).left();
    assert_eq!(whole, usize::try_from(u32::MAX).unwrap(), "the whole cap");
}
