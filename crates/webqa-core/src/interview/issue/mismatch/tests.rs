//! Tests of the rulings on the mismatch questions.

use super::*;
use crate::interview::answer::{Reply, read_mismatch};
use crate::interview::issue::tests::fixture::{
    CLICK1, CUT, GOAL, IN1, answered, cart, cart_purpose, clicked, confirm, count, guessing,
    product, products, purpose_match, questions, record, run, screen, seen, sure,
};
use crate::interview::issue::tests::sample::mismatch_of;
use crate::interview::issue::{Act, Count, Seen, step_issues};
use crate::interview::outcome::Outcome;

/// The ruling `answer` on the question of `/products/*` over `records`.
fn ruled_on(records: &[Record<'_>], answer: Option<Match>) -> Ruling {
    let screens = [products()];
    ruling(&questions(&screens, records)[0], answer, &run(records))
}

/// What `f` makes of the question of `/products/*` and the run of one
/// finished trial per guess, each of one step at `/products/42`.
fn guessed<R>(guesses: &[&str], f: impl FnOnce(&MismatchQuestion<'_>, &Run<'_>) -> R) -> R {
    let page = product(42);
    let replies: Vec<Reply> = guesses.iter().map(|guess| guessing(guess)).collect();
    let steps: Vec<[Seen<'_>; 1]> = replies
        .iter()
        .map(|reply| [answered(&page, reply)])
        .collect();
    let records: Vec<Record<'_>> = steps.iter().map(|steps| record(GOAL, steps)).collect();
    let screens = [products()];
    f(&questions(&screens, &records)[0], &run(&records))
}

/// The ruling `answer` on the question of `guesses`.
fn ruled(guesses: &[&str], answer: Option<Match>) -> Ruling {
    guessed(guesses, |q, run| ruling(q, answer, run))
}

const THREE: [&str; 3] = ["会員登録をする", "会員登録をする", "商品をカートに入れる"];

/// Step 1 of trial 1.
const FIRST: Shot = Shot { trial: 1, step: 1 };

/// The mismatch issue of `guesses` in `count`, first seen at `shot` after
/// `path`.
fn mismatch(guesses: &[&str], count: Count, shot: Shot, path: Vec<Act>) -> Ruling {
    Ruling::Mismatch(Box::new(mismatch_of(guesses, count, shot, path)))
}

/// The guesses passed to the question of [`THREE`].
const ASKED: [&str; 2] = ["会員登録をする", "商品をカートに入れる"];

#[test]
fn judge_says_different_purpose_is_mismatch() {
    let asked = guessed(&THREE, |q, _| q.guesses.clone());
    assert_eq!(asked, ASKED, "the guesses asked");
    assert_eq!(
        ruled(&THREE, Some(Match::Different)),
        mismatch(&ASKED, count(3, 3, 3), FIRST, vec![]),
        "a mismatch issue"
    );
}

#[test]
fn mismatch_issue_screen_is_url_pattern() {
    let number = Number::of(Key {
        spot: Spot::Pattern("/products/*"),
        kind: Kind::Mismatch,
    });
    let expected = Issue {
        number,
        place: Place::Pattern("/products/*".to_owned()),
        target: None,
        confidence: None,
        ..mismatch_of(&ASKED, count(3, 3, 3), FIRST, vec![])
    };
    let got = ruled(&THREE, Some(Match::Different));
    assert_eq!(
        got,
        Ruling::Mismatch(Box::new(expected.clone())),
        "the issue"
    );
    assert_eq!(expected.kind(), Kind::Mismatch, "the kind");
    assert_eq!(number.to_string(), "I-D0D06221", "the number");
}

#[test]
fn judge_says_same_purpose_no_mismatch() {
    let got = ruled(&["この商品を買い物かごに追加する"], Some(Match::Same));
    assert_eq!(got, Ruling::Same, "the judge said same");
    let written = ["商品をカートに入れる"];
    assert_eq!(
        ruled(&written, Some(Match::Different)),
        mismatch(&written, count(1, 1, 1), FIRST, vec![]),
        "not compared as text"
    );
}

#[test]
fn undeclared_screen_not_compared() {
    let help = screen("/help", &[("link", "問い合わせ", IN1)]);
    let guess = guessing("問い合わせをする");
    let steps = [answered(&help, &guess)];
    let records = [record(GOAL, &steps)];
    let screens = [products(), cart_purpose()];
    for (i, screens) in [&screens[..1], &screens, &[]].into_iter().enumerate() {
        assert_eq!(questions(screens, &records), vec![], "case {i}");
    }
    let issues = step_issues(&run(&records));
    assert_eq!(issues, vec![], "the steps do not read the guesses");
}

#[test]
fn unusable_mismatch_answer_goes_to_unchecked() {
    let expected = UncheckedPurpose {
        pattern: "/products/*".to_owned(),
        purpose: "商品をカートに入れる".to_owned(),
    };
    let tables = [Some(&[("たぶん", 0.9)][..]), None];
    let read = tables.map(|table| read_mismatch(&purpose_match(table)).ok());
    for (i, answer) in read.into_iter().chain([None]).enumerate() {
        let got = ruled(&THREE, answer);
        assert_eq!(got, Ruling::Unchecked(expected.clone()), "case {i}");
    }
    assert_eq!(guessed(&THREE, |q, _| unchecked(q)), expected, "the entry");
}

/// The guess of [`four_trials`] passed to the question.
const JOIN: [&str; 1] = ["会員登録をする"];

/// The ruling `different` on four trials, the first ending as `first`:
/// it reaches `/products/42` at step 4 after a click; the second never
/// reaches the pattern; the third reaches `/products/43` at step 2; the
/// fourth, cut, reaches `/products/44`.
fn four_trials(first: Outcome) -> Ruling {
    let (cart, confirm, sure) = (cart(), confirm(), sure());
    let [p42, p43, p44] = [42, 43, 44].map(product);
    let (join, compare) = (guessing("会員登録をする"), guessing("商品を比べる"));
    let at_cart = answered(&cart, &sure);
    let one = [
        seen(&cart, Some(&sure), &[], CLICK1),
        at_cart,
        at_cart,
        answered(&p42, &join),
    ];
    let two = [at_cart, answered(&confirm, &sure)];
    let three = [at_cart, answered(&p43, &join)];
    let four = [answered(&p44, &compare)];
    let records = [
        record(first, &one),
        record(GOAL, &two),
        record(GOAL, &three),
        record(CUT, &four),
    ];
    ruled_on(&records, Some(Match::Different))
}

#[test]
fn mismatch_issue_counts_trials_reaching_pattern() {
    assert_eq!(
        four_trials(GOAL),
        mismatch(&JOIN, count(2, 2, 2), Shot { trial: 1, step: 4 }, clicked()),
        "two of three reached"
    );
}

#[test]
fn mismatch_screenshot_from_first_matching_observation() {
    assert_eq!(
        four_trials(GOAL),
        mismatch(&JOIN, count(2, 2, 2), Shot { trial: 1, step: 4 }, clicked()),
        "the first trial's step 4"
    );
    assert_eq!(
        four_trials(CUT),
        mismatch(&JOIN, count(1, 1, 1), Shot { trial: 3, step: 2 }, vec![]),
        "the first trial cut"
    );
}
