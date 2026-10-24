//! Materials of the naturalness tests: the persona, the goal, answers,
//! caps, mismatch questions, and readings of the questions asked.

use std::num::NonZeroU32;

pub use crate::config::persona::tests::persona;
use crate::config::purpose::{ScreenPurpose, UrlPattern};
use crate::interview::ask::{
    EXPECTATION, HARD_TO_SEE, HESITATION, MismatchQuestion, PURPOSE_GUESS, PURPOSE_MATCH, preamble,
};
use crate::interview::issue::Issue;
use crate::interview::issue::tests::fixture::{
    GOAL as REACHED, answered, cart, cart_purpose, guessing, product, products, questions, record,
    shipping,
};
use crate::interview::natural::{NATURAL, PERSONA_TRAIT, Quota, natural_questions};
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

/// The description of a sample issue.
pub const HESITANT: &str = "試行をすべて終えたあと、次の課題をまとめた。
種類: 判断に迷う
画面: https://shop.example.test/confirm
要素: button \"次へ\"
手がかり: 迷いの度合いが閾値以上、引き返した
期待: 押すと注文が確定すると思った
起きた回数: 3回の試行のうち2回";

/// The description of a sample issue.
pub const VISIBILITY: &str = "試行をすべて終えたあと、次の課題をまとめた。
種類: 視認性が悪い
画面: https://shop.example.test/shipping
要素: button \"クーポンを使う\"
見えにくい理由: 文字が薄い
期待: クーポンを入力できると思った
起きた回数: 3回の試行のうち1回";

/// The description of a sample issue.
pub const UNREACHED: &str = "試行をすべて終えたあと、次の課題をまとめた。
種類: 目的に届かない
画面: https://shop.example.test/shipping
要素: なし
終わり方: ペルソナが諦めた
期待: 次に進むボタンが見つからない
起きた回数: 3回の試行のうち1回";

/// The description of a sample issue.
pub const MISMATCH: &str = "試行をすべて終えたあと、次の課題をまとめた。
種類: 目的のずれ
画面: URL の型 /products/*
要素: なし
書いた目的: 商品をカートに入れる
推測: 会員登録をする、商品をカートに入れる
起きた回数: 2回の試行のうち2回";

/// The two naturalness questions about an issue described as `d`.
pub fn expected_questions(d: &str) -> Vec<Question> {
    let lead = format!("{}\n\n{d}", preamble(&persona(), GOAL));
    vec![
        Question::Choice {
            id: QuestionId::new("natural"),
            prompt: format!("{lead}\n\nこのペルソナが、この場面でこう考え、こう操作するのは自然か"),
            options: vec!["自然".to_owned(), "自然でない".to_owned()],
        },
        Question::Choice {
            id: QuestionId::new("persona_trait"),
            prompt: format!(
                "{lead}\n\nこの課題は、ペルソナのどの性質から起きたと考えるかを1つ選ぶ"
            ),
            options: ["どんな人か", "ITの慣れ", "達成したいこと", "しないこと"]
                .map(str::to_owned)
                .to_vec(),
        },
    ]
}
