//! Tests of reading the answers to the naturalness questions, and of the
//! sections they put the issues in.

use super::*;
use crate::interview::issue::tests::sample::sample;
use crate::interview::natural::tests::fixture::{answers, both, persona};
use crate::interview::natural::{Placed, Section, Trait, place};

const NAN: f64 = f64::NAN;

/// The trait with the persona's text of it.
fn of(which: Trait) -> PersonaTrait {
    PersonaTrait {
        which,
        content: which.content(&persona()),
    }
}

/// The answers to the two questions, the naturalness read, and the trait.
type Read = (
    &'static [(&'static str, f64)],
    &'static [(&'static str, f64)],
    bool,
    Trait,
);

const READ: [Read; 3] = [
    (
        &[("自然", 1.0)],
        &[("しないこと", 0.6), ("ITの慣れ", 0.4)],
        true,
        Trait::Avoids,
    ),
    (
        &[("自然", 0.5), ("自然でない", 0.5)],
        &[("どんな人か", 0.7)],
        true,
        Trait::Profile,
    ),
    (
        &[("自然", NAN), ("自然でない", 0.1)],
        &[("達成したいこと", 0.0)],
        false,
        Trait::Wants,
    ),
];

#[test]
fn natural_answer_goes_to_body() {
    let read = read_natural(
        &both(&[("自然", 0.8), ("自然でない", 0.2)], &[("ITの慣れ", 0.9)]),
        &persona(),
    );
    let it_literacy = PersonaTrait {
        which: Trait::ItLiteracy,
        content: "スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている"
            .to_owned(),
    };
    let expected = Judgement {
        natural: true,
        persona_trait: it_literacy.clone(),
    };
    assert_eq!(read, Ok(expected), "natural, from IT literacy");
    let placed = place(sample("I-00000001", 2, 3), read.ok());
    let body = Placed {
        issue: sample("I-00000001", 2, 3),
        section: Section::Body,
        persona_trait: Some(it_literacy),
    };
    assert_eq!(placed, body, "in the body with its trait");
    for (i, (natural, trait_, yes, which)) in READ.into_iter().enumerate() {
        let got = read_natural(&both(natural, trait_), &persona());
        let expected = Judgement {
            natural: yes,
            persona_trait: of(which),
        };
        assert_eq!(got, Ok(expected), "case {i}");
    }
}

#[test]
fn unnatural_answer_goes_to_off_persona() {
    let read = read_natural(
        &both(&[("自然でない", 0.7), ("自然", 0.3)], &[("ITの慣れ", 0.9)]),
        &persona(),
    );
    let expected = Judgement {
        natural: false,
        persona_trait: of(Trait::ItLiteracy),
    };
    assert_eq!(read, Ok(expected), "not natural");
    let placed = place(sample("I-00000001", 2, 3), read.ok());
    let off = Placed {
        issue: sample("I-00000001", 2, 3),
        section: Section::OffPersona,
        persona_trait: Some(of(Trait::ItLiteracy)),
    };
    assert_eq!(placed, off, "off the persona, keeping its trait");
}

/// Makes the problem of a question from its name.
type Variant = fn(QuestionId) -> Unusable;

/// Answers as `(question, options)` rows.
type Rows = &'static [(&'static str, &'static [(&'static str, f64)])];

/// A usable answer to the naturalness question.
const USABLE: &[(&str, f64)] = &[("自然", 0.9)];

/// Answers with the problem expected and the question it is about.
const UNUSABLE: [(Rows, Variant, &str); 12] = [
    (
        &[(PERSONA_TRAIT, &[("ITの慣れ", 0.9)])],
        Unusable::Missing,
        NATURAL,
    ),
    (&[(NATURAL, &[])], Unusable::Missing, NATURAL),
    (&[(NATURAL, &[("自然", NAN)])], Unusable::Missing, NATURAL),
    (
        &[(NATURAL, &[("たぶん", 0.9)])],
        Unusable::NotAnOption,
        NATURAL,
    ),
    (
        &[(NATURAL, &[(" 自然", 0.9)])],
        Unusable::NotAnOption,
        NATURAL,
    ),
    (
        &[(NATURAL, &[("自然", 0.4), ("自然", 0.6)])],
        Unusable::NotAnOption,
        NATURAL,
    ),
    (
        &[(NATURAL, &[("自然", 1.5)])],
        Unusable::OutOfRange,
        NATURAL,
    ),
    (
        &[(NATURAL, &[("たぶん", 1.5)])],
        Unusable::NotAnOption,
        NATURAL,
    ),
    (&[(NATURAL, USABLE)], Unusable::Missing, PERSONA_TRAIT),
    (
        &[(NATURAL, USABLE), (PERSONA_TRAIT, &[("名前", 0.9)])],
        Unusable::NotAnOption,
        PERSONA_TRAIT,
    ),
    (
        &[(NATURAL, USABLE), (PERSONA_TRAIT, &[("ITの慣れ", -0.1)])],
        Unusable::OutOfRange,
        PERSONA_TRAIT,
    ),
    (
        &[(NATURAL, &[("たぶん", 0.9)])],
        Unusable::NotAnOption,
        NATURAL,
    ),
];

#[test]
fn unusable_answer_goes_to_unchecked() {
    let unchecked = Placed {
        issue: sample("I-00000001", 2, 3),
        section: Section::Unchecked,
        persona_trait: None,
    };
    for (i, (rows, variant, id)) in UNUSABLE.into_iter().enumerate() {
        let read = read_natural(&answers(rows), &persona());
        assert_eq!(read, Err(variant(QuestionId::new(id))), "case {i}");
        let placed = place(sample("I-00000001", 2, 3), read.ok());
        assert_eq!(placed, unchecked, "case {i}");
    }
    let message =
        read_natural(&both(&[("たぶん", 0.9)], &[]), &persona()).map_err(|e| e.to_string());
    assert_eq!(
        message,
        Err("判断モデルの答えの natural が問いの選択肢に無い".to_owned()),
        "the name of the question, not the answer"
    );
    let unasked = place(sample("I-00000001", 2, 3), None);
    assert_eq!(unasked, unchecked, "not asked, or the judge failed");
}

#[test]
fn sections_follow_ledger_order() {
    let order = [Section::Body, Section::Unchecked, Section::OffPersona];
    let mut sorted = [Section::OffPersona, Section::Body, Section::Unchecked];
    sorted.sort();
    assert_eq!(sorted, order, "body, unchecked, off the persona");
    let ascending = (
        Section::Body < Section::Unchecked,
        Section::Unchecked < Section::OffPersona,
    );
    assert_eq!(ascending, (true, true), "in the ledger's order");
}
