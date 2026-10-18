//! Tests of reading the answers of an interview step.

mod cases;

use std::collections::BTreeMap;

use super::*;
use crate::config::raw::Op;
use crate::interview::ask::tests::{shot, step};
use crate::interview::ask::{
    DIFFERENT, EXPECTATION, HARD_TO_SEE, HESITATION, PURPOSE_GUESS, PURPOSE_MATCH, SAME,
};
use crate::judge::{QuestionId, Usage};

const NAN: f64 = f64::NAN;

/// The options of one answer, each with its probability.
type Table<'a> = &'a [(&'a str, f64)];

/// Answers changed from the full ones: each name with its new options, or
/// `None` to leave the question unanswered.
type Changes<'a> = &'a [(&'a str, Option<Table<'a>>)];

/// The variant of the problem expected, with the name of its question.
type Expected = (fn(QuestionId) -> Unusable, &'static str);

const FULL: [(&str, Table<'_>); 7] = [
    ("operation", &[("click", 0.9)]),
    ("target", &[("1", 0.8)]),
    ("error_shown", &[("true", 0.1)]),
    ("goal_reached", &[("true", 0.0)]),
    (HESITATION, &[("true", 0.7)]),
    (EXPECTATION, &[("押すと注文が確定すると思った", NAN)]),
    (PURPOSE_GUESS, &[("商品をカートに入れる", NAN)]),
];

const HARD: (&str, Table<'_>) = (HARD_TO_SEE, &[("2: 文字が薄い", NAN)]);

fn row((name, table): (&str, Table<'_>)) -> (QuestionId, Vec<(String, f64)>) {
    let options = table.iter().map(|(o, p)| ((*o).to_owned(), *p)).collect();
    (QuestionId::new(name), options)
}

/// Every answer, with the elements hard to see when `image` is set.
fn full(image: bool) -> Answers {
    let rows = FULL.into_iter().chain(image.then_some(HARD));
    Answers {
        table: rows.map(row).collect::<BTreeMap<_, _>>(),
        usage: Usage::default(),
    }
}

/// `answers` with `changes` made.
fn changed(mut answers: Answers, changes: Changes<'_>) -> Answers {
    for &(name, table) in changes {
        answers.table.remove(&QuestionId::new(name));
        answers.table.extend(table.map(|table| row((name, table))));
    }
    answers
}

/// Checks each change of `full(true)` against the problem expected.
fn rejects(step: &Step<'_>, cases: &[(Changes<'_>, Expected)]) {
    for (i, &(changes, (variant, name))) in cases.iter().enumerate() {
        let answers = changed(full(true), changes);
        let got = read_step(&answers, step);
        assert_eq!(got, Err(variant(QuestionId::new(name))), "case {i}");
    }
}

fn verdict() -> Verdict {
    Verdict {
        op: Op::Click,
        confidence: Some(0.9),
        targets: vec![1],
        error_shown: Some(0.1),
        goal_reached: Some(0.0),
        usage: Usage::default(),
    }
}

const VERBATIM: [&str; 3] = ["  前後に空白  ", "一行目\n二行目", "「注文」を押す"];

#[test]
fn free_text_kept_verbatim() {
    let reply = Reply {
        verdict: verdict(),
        hesitation: 0.7,
        expectation: "押すと注文が確定すると思った".to_owned(),
        hard_to_see: None,
        purpose_guess: "商品をカートに入れる".to_owned(),
    };
    assert_eq!(
        read_step(&full(false), &step(None)),
        Ok(reply),
        "as written"
    );
    for text in VERBATIM {
        for name in [EXPECTATION, PURPOSE_GUESS] {
            let answers = changed(full(false), &[(name, Some(&[(text, 0.3)]))]);
            let reply = read_step(&answers, &step(None));
            let got = reply.map(|r| [r.expectation, r.purpose_guess]);
            let both = full_texts(name, text);
            assert_eq!(got, Ok(both), "{name} {text:?}");
        }
    }
}

/// The expectation and the purpose guess, with the one named `name`
/// replaced by `text`.
fn full_texts(name: &str, text: &str) -> [String; 2] {
    let texts = [
        (EXPECTATION, "押すと注文が確定すると思った"),
        (PURPOSE_GUESS, "商品をカートに入れる"),
    ];
    texts.map(|(id, written)| if id == name { text } else { written }.to_owned())
}

#[test]
fn hard_to_see_optional_without_image_support() {
    let far = changed(full(true), &[(HARD_TO_SEE, Some(&[("99: x", NAN)]))]);
    for answers in [full(false), full(true), far] {
        let got = read_step(&answers, &step(None)).map(|r| r.hard_to_see);
        assert_eq!(got, Ok(None), "not asked, not read");
    }
}

#[test]
fn added_ids_are_named() {
    assert_eq!(
        [
            HESITATION,
            EXPECTATION,
            HARD_TO_SEE,
            PURPOSE_GUESS,
            PURPOSE_MATCH
        ],
        [
            "hesitation",
            "expectation",
            "hard_to_see",
            "purpose_guess",
            "purpose_match"
        ],
        "the names of the added questions"
    );
    assert_eq!([SAME, DIFFERENT], ["同じ", "同じでない"], "the two choices");
}
