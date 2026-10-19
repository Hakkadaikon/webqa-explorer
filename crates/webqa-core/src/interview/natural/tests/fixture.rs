//! Materials of the naturalness tests: the persona, the goal, answers,
//! caps, mismatch questions, and readings of the questions asked.

use std::num::NonZeroU32;

pub use crate::config::persona::tests::persona;
use crate::config::purpose::{ScreenPurpose, UrlPattern};
use crate::interview::ask::{
    EXPECTATION, HARD_TO_SEE, HESITATION, MismatchQuestion, PURPOSE_GUESS, PURPOSE_MATCH,
};
use crate::interview::issue::Issue;
use crate::interview::issue::tests::fixture::{
    GOAL as REACHED, answered, cart, cart_purpose, guessing, product, products, questions, record,
    shipping,
};
use crate::interview::natural::{
    Judgement, NATURAL, PERSONA_TRAIT, PersonaTrait, Quota, Trait, natural_questions,
};
use crate::judge::{Answers, Question, QuestionId, Usage};

/// The goal of the step the persona aims at.
pub const GOAL: &str = "注文を確定できる画面まで進む";

/// The answers of `rows`: each question's options with their probabilities.
pub fn answers(rows: &[(&str, &[(&str, f64)])]) -> Answers {
    let table = rows
        .iter()
        .map(|&(id, options)| {
            let options = options.iter().map(|&(o, p)| (o.to_owned(), p)).collect();
            (QuestionId::new(id), options)
        })
        .collect();
    Answers {
        table,
        usage: Usage::default(),
    }
}

/// The answers to the two naturalness questions.
pub fn both(natural: &[(&str, f64)], trait_: &[(&str, f64)]) -> Answers {
    answers(&[(NATURAL, natural), (PERSONA_TRAIT, trait_)])
}

/// The cap of `n` questions.
pub fn quota(n: u32) -> Quota {
    Quota::new(NonZeroU32::new(n).unwrap())
}

/// The screen purpose of `/shipping`.
fn shipping_purpose() -> ScreenPurpose {
    ScreenPurpose {
        pattern: UrlPattern::parse("/shipping").unwrap(),
        purpose: "配送先を入力する".to_owned(),
    }
}

/// The screen purposes of `/products/*`, `/cart` and `/shipping`.
pub fn three_purposes() -> Vec<ScreenPurpose> {
    vec![products(), cart_purpose(), shipping_purpose()]
}

/// The mismatch questions of `screens` from one trial that reached the
/// goal through a product, the cart and shipping, guessing at each.
pub fn mismatch_of(screens: &[ScreenPurpose]) -> Vec<MismatchQuestion<'_>> {
    let (product, cart, shipping) = (product(42), cart(), shipping());
    let guessed = guessing("会員登録をする");
    let steps = [
        answered(&product, &guessed),
        answered(&cart, &guessed),
        answered(&shipping, &guessed),
    ];
    questions(screens, &[record(REACHED, &steps)])
}

/// The numbers of `issues`, in order.
pub fn numbers(issues: &[Issue]) -> Vec<String> {
    issues.iter().map(|it| it.number.to_string()).collect()
}

/// The URL patterns of `questions`, in order.
pub fn patterns<'a>(questions: &'a [MismatchQuestion<'_>]) -> Vec<&'a str> {
    questions
        .iter()
        .map(|q| q.screen.pattern.as_str())
        .collect()
}

/// The names of the questions of a step.
fn step_ids() -> Vec<QuestionId> {
    let asked = [
        HESITATION,
        EXPECTATION,
        HARD_TO_SEE,
        PURPOSE_GUESS,
        PURPOSE_MATCH,
    ];
    let mut ids = vec![
        QuestionId::operation(),
        QuestionId::target(),
        QuestionId::error_shown(),
        QuestionId::goal_reached(),
    ];
    ids.extend(asked.map(QuestionId::new));
    ids
}

/// The names of the naturalness questions about `issue`, and those of
/// them a step also asks.
pub fn ids_of(issue: &Issue) -> (Vec<String>, Vec<String>) {
    let questions = natural_questions(&persona(), GOAL, issue);
    let ids: Vec<&QuestionId> = questions.iter().map(Question::id).collect();
    let step = step_ids();
    let shared = ids.iter().filter(|&&id| step.contains(id)).copied();
    (names(ids.iter().copied()), names(shared))
}

fn names<'a>(ids: impl Iterator<Item = &'a QuestionId>) -> Vec<String> {
    ids.map(|id| id.as_str().to_owned()).collect()
}

/// Whether the prompt of the first naturalness question about `issue`
/// shows each of `lines`.
pub fn shows(issue: &Issue, lines: [&str; 2]) -> [bool; 2] {
    let questions = natural_questions(&persona(), GOAL, issue);
    let prompt = questions
        .first()
        .map(|q| format!("{q:?}"))
        .unwrap_or_default();
    lines.map(|line| prompt.contains(line))
}

/// The judge's answer that the issue is not natural, from IT literacy.
pub fn unnatural() -> Judgement {
    Judgement {
        natural: false,
        persona_trait: PersonaTrait {
            which: Trait::ItLiteracy,
            content: Trait::ItLiteracy.content(&persona()),
        },
    }
}
