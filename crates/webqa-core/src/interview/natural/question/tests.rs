//! Tests of the naturalness questions and the names of the traits.

use super::*;
use crate::config::persona::Persona;
use crate::interview::ask::preamble;
use crate::interview::issue::tests::fixture::next1;
use crate::interview::issue::tests::sample::{
    mismatch_sample, sample, unreached_sample, visibility_sample,
};
use crate::interview::issue::{Detail, Halt, Target};
use crate::interview::natural::tests::fixture::{GOAL, persona};
use crate::interview::natural::{NATURAL, NO, PERSONA_TRAIT, TRAITS, Trait, YES};

const HESITANT: &str = "試行をすべて終えたあと、次の課題をまとめた。
種類: 判断に迷う
画面: https://shop.example.test/confirm
要素: button \"次へ\"
手がかり: 迷いの度合いが閾値以上、引き返した
期待: 押すと注文が確定すると思った
起きた回数: 3回の試行のうち2回";

const VISIBILITY: &str = "試行をすべて終えたあと、次の課題をまとめた。
種類: 視認性が悪い
画面: https://shop.example.test/shipping
要素: button \"クーポンを使う\"
見えにくい理由: 文字が薄い
期待: クーポンを入力できると思った
起きた回数: 3回の試行のうち1回";

const UNREACHED: &str = "試行をすべて終えたあと、次の課題をまとめた。
種類: 目的に届かない
画面: https://shop.example.test/shipping
要素: なし
終わり方: ペルソナが諦めた
期待: 次に進むボタンが見つからない
起きた回数: 3回の試行のうち1回";

const MISMATCH: &str = "試行をすべて終えたあと、次の課題をまとめた。
種類: 目的のずれ
画面: URL の型 /products/*
要素: なし
書いた目的: 商品をカートに入れる
推測: 会員登録をする、商品をカートに入れる
起きた回数: 2回の試行のうち2回";

/// The unreached sample whose last step's answers were unusable.
fn unreached_unanswered() -> Issue {
    Issue {
        detail: Detail::Unreached {
            halt: Halt::GaveUp,
            expectation: None,
        },
        ..unreached_sample()
    }
}

/// The hesitation sample at an element whose name breaks the line.
fn broken_name() -> Issue {
    let target = Target {
        name: "次\nへ".to_owned(),
        ..next1()
    };
    Issue {
        target: Some(target),
        ..sample("I-5334A6B3", 2, 3)
    }
}

/// The two questions about the hesitation sample.
fn hesitant_questions() -> Vec<Question> {
    let lead = format!("{}\n\n{HESITANT}", preamble(&persona(), GOAL));
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

#[test]
fn natural_question_describes_issue() {
    let described = [
        (sample("I-5334A6B3", 2, 3), HESITANT.to_owned()),
        (visibility_sample(), VISIBILITY.to_owned()),
        (unreached_sample(), UNREACHED.to_owned()),
        (
            unreached_unanswered(),
            UNREACHED.replace("期待: 次に進むボタンが見つからない", "期待: なし"),
        ),
        (mismatch_sample(), MISMATCH.to_owned()),
    ];
    for (issue, expected) in described {
        assert_eq!(describe(&issue), expected, "{issue:?}");
    }
    let line = describe(&broken_name()).lines().nth(3).map(str::to_owned);
    let expected = "要素: button \"次\\nへ\"".to_owned();
    assert_eq!(line, Some(expected), "the name quoted on one line");
    let got = natural_questions(&persona(), GOAL, &sample("I-5334A6B3", 2, 3));
    assert_eq!(got, hesitant_questions(), "the two questions");
}

/// Each trait with the persona's text of it.
const CONTENTS: [(Trait, &str); 4] = [
    (
        Trait::Profile,
        "60代。孫に頼まれて、週末にスマートフォンで買い物をする",
    ),
    (
        Trait::ItLiteracy,
        "スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている",
    ),
    (Trait::Wants, "孫の誕生日プレゼントを注文する"),
    (Trait::Avoids, "URL を手で打たない、開発者ツールを開かない"),
];

/// The profile and the avoids of a persona with a spaced profile and
/// nothing to avoid.
fn spaced() -> (String, String) {
    let spaced = Persona {
        profile: "  60代。\n".to_owned(),
        avoids: vec![],
        ..persona()
    };
    (
        Trait::Profile.content(&spaced),
        Trait::Avoids.content(&spaced),
    )
}

#[test]
fn trait_labels_follow_preamble() {
    let names = TRAITS.map(|(_, name)| name);
    let expected = ["どんな人か", "ITの慣れ", "達成したいこと", "しないこと"];
    assert_eq!(names, expected, "the names in order");
    let labels = TRAITS.map(|(which, _)| which.label());
    assert_eq!(labels, expected, "the label of each trait");
    let lead = preamble(&persona(), GOAL);
    let shown = names.map(|name| lead.contains(&format!("{name}:")));
    assert_eq!(shown, [true; 4], "{lead}");
    for (which, content) in CONTENTS {
        assert_eq!(which.content(&persona()), content, "{which:?}");
    }
    assert_eq!(
        spaced(),
        ("60代。".to_owned(), String::new()),
        "trimmed, empty"
    );
    let ids = [NATURAL, PERSONA_TRAIT, YES, NO];
    let expected = ["natural", "persona_trait", "自然", "自然でない"];
    assert_eq!(ids, expected, "the names of the questions and options");
}
