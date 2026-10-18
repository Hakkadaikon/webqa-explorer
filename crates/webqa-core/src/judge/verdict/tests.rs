//! Tests of reading the answers of a step.
//!
//! A row of answers is written as `"<question> <option>:<p> …"`, such as
//! `"target 1:0.8 2:NaN"`; a question alone has no option.

mod reads;
mod rejects;

use std::collections::BTreeMap;

use super::*;
use crate::judge::JudgeError;
use crate::judge::ask::tests::ASK8;

/// The answers of [`ASK8`] that every test changes one row of.
const GOOD: [&str; 4] = [
    "operation click:0.9 back:0.1",
    "target 1:0.8 2:0.6",
    "error_shown true:0.1",
    "goal_reached true:0.0",
];

const NO_GOAL: Ask<'static> = Ask { goal: None, ..ASK8 };
const NO_ELEMENT: Ask<'static> = Ask {
    elements: 0,
    ..ASK8
};
const MANY: Ask<'static> = Ask {
    elements: 300,
    ..ASK8
};

/// The question and its options of a row.
fn row(row: &str) -> (QuestionId, Vec<(String, f64)>) {
    let mut words = row.split_whitespace();
    let id = QuestionId::new(words.next().unwrap());
    let options = words
        .map(|o| o.rsplit_once(':').unwrap())
        .map(|(name, p)| (name.to_owned(), p.parse().unwrap()))
        .collect();
    (id, options)
}

/// [`GOOD`] with each of `rows` in place of the row of the same question.
fn with(rows: &[&str]) -> Answers {
    let mut table: BTreeMap<_, _> = GOOD.iter().map(|r| row(r)).collect();
    table.extend(rows.iter().map(|r| row(r)));
    Answers {
        table,
        usage: Usage {
            input_tokens: 120,
            output_tokens: 3,
        },
    }
}

/// [`GOOD`] without the row of `id`.
fn without(id: &str) -> Answers {
    let mut answers = with(&[]);
    answers.table.remove(&QuestionId::new(id));
    answers
}

#[test]
fn judge_messages_carry_no_answer() {
    let shown = [
        Unusable::Missing(QuestionId::operation()),
        Unusable::NotAnOption(QuestionId::target()),
        Unusable::OutOfRange(QuestionId::error_shown()),
    ]
    .map(|u| u.to_string());
    let want = [
        "判断モデルの答えに operation が無い",
        "判断モデルの答えの target が問いの選択肢に無い",
        "判断モデルの答えの error_shown の確率が 0 以上 1 以下でない",
    ];
    assert_eq!(shown, want, "fixed messages with the question's name");
    let errors = [
        JudgeError::Api,
        JudgeError::Timeout,
        JudgeError::Malformed,
        JudgeError::Bridge,
    ]
    .map(|e| e.to_string());
    let want = [
        "判断モデルの API がエラーを返した",
        "判断モデルの答えが待ち時間内に返らなかった",
        "判断モデルの答えの形式が崩れている",
        "橋渡しのコマンドとやりとりできない",
    ];
    assert_eq!(errors, want, "fixed messages");
    let drag = read_answers(&with(&["operation drag:0.9"]), &ASK8).unwrap_err();
    let shown = format!("{drag} {drag:?}");
    assert!(!shown.contains("drag"), "no answer in {shown}");
}
