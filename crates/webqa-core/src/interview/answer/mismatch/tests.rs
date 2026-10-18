//! Tests of reading the answer to a mismatch question.

use super::*;
use crate::judge::Usage;

const NAN: f64 = f64::NAN;

/// The options of the answer, or `None` for no answer.
type Table<'a> = Option<&'a [(&'a str, f64)]>;

fn answers(table: Table<'_>) -> Answers {
    let rows = table.map(|table| {
        let options = table.iter().map(|(o, p)| ((*o).to_owned(), *p)).collect();
        (QuestionId::new(PURPOSE_MATCH), options)
    });
    Answers {
        table: rows.into_iter().collect(),
        usage: Usage::default(),
    }
}

/// Makes the problem of a question from its name.
type Variant = fn(QuestionId) -> Unusable;

/// Answers with the variant of the problem expected.
const UNUSABLE: [(Table<'_>, Variant); 9] = [
    (None, Unusable::Missing),
    (Some(&[]), Unusable::Missing),
    (
        Some(&[("同じ", NAN), ("同じでない", NAN)]),
        Unusable::Missing,
    ),
    (Some(&[("たぶん", 0.9)]), Unusable::NotAnOption),
    (Some(&[("同じ", 0.2), ("違う", 0.8)]), Unusable::NotAnOption),
    (Some(&[("同じ", 0.4), ("同じ", 0.6)]), Unusable::NotAnOption),
    (Some(&[(" 同じ", 0.9)]), Unusable::NotAnOption),
    (Some(&[("同じ", 1.5)]), Unusable::OutOfRange),
    (Some(&[("たぶん", 1.5)]), Unusable::NotAnOption),
];

#[test]
fn mismatch_answer_missing_or_off_choice_is_unusable() {
    for (i, (table, variant)) in UNUSABLE.into_iter().enumerate() {
        let got = read_mismatch(&answers(table));
        assert_eq!(
            got,
            Err(variant(QuestionId::new(PURPOSE_MATCH))),
            "case {i}"
        );
    }
    let message = read_mismatch(&answers(Some(&[("たぶん", 0.9)]))).map_err(|e| e.to_string());
    assert_eq!(
        message,
        Err("判断モデルの答えの purpose_match が問いの選択肢に無い".to_owned()),
        "the name of the question, not the answer"
    );
}

const READ: [(&[(&str, f64)], Match); 6] = [
    (&[("同じ", 0.8), ("同じでない", 0.2)], Match::Same),
    (&[("同じ", 0.3), ("同じでない", 0.7)], Match::Different),
    (&[("同じでない", 1.0)], Match::Different),
    (&[("同じ", 0.0)], Match::Same),
    (&[("同じ", NAN), ("同じでない", 0.1)], Match::Different),
    (&[("同じでない", 0.5), ("同じ", 0.5)], Match::Different),
];

#[test]
fn mismatch_answer_reads_same_or_different() {
    for (table, want) in READ {
        let got = read_mismatch(&answers(Some(table)));
        assert_eq!(got, Ok(want), "{table:?}");
    }
}
