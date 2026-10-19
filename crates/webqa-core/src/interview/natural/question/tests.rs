//! Tests of the naturalness questions and the names of the traits.

use super::*;
use crate::config::persona::Persona;
use crate::interview::ask::preamble;
use crate::interview::issue::tests::fixture::next1;
use crate::interview::issue::tests::sample::{
    mismatch_sample, sample, unreached_sample, visibility_sample,
};
use crate::interview::issue::{Detail, Halt, Target};
use crate::interview::natural::tests::fixture::{
    GOAL, HESITANT, MISMATCH, UNREACHED, VISIBILITY, expected_questions, persona,
};
use crate::interview::natural::{NATURAL, NO, PERSONA_TRAIT, TRAITS, Trait, YES};

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
    assert_eq!(got, expected_questions(HESITANT), "the two questions");
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

/// The text of each trait of a persona whose texts have spaces and line
/// breaks around them.
fn spaced() -> [String; 4] {
    let spaced = Persona {
        profile: "  60代。\n".to_owned(),
        it_literacy: " スマートフォンの基本操作だけ\n".to_owned(),
        wants: "\t孫の誕生日プレゼントを注文する ".to_owned(),
        avoids: vec![
            " URL を手で打たない\n".to_owned(),
            "  開発者ツールを開かない ".to_owned(),
        ],
        ..persona()
    };
    TRAITS.map(|(which, _)| which.content(&spaced))
}

/// The text of the avoids of a persona with nothing to avoid.
fn nothing_avoided() -> String {
    let none = Persona {
        avoids: vec![],
        ..persona()
    };
    Trait::Avoids.content(&none)
}

/// The trimmed texts of [`spaced`].
const TRIMMED: [&str; 4] = [
    "60代。",
    "スマートフォンの基本操作だけ",
    "孫の誕生日プレゼントを注文する",
    "URL を手で打たない、開発者ツールを開かない",
];

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
    assert_eq!(spaced(), TRIMMED, "trimmed");
    assert_eq!(nothing_avoided(), "", "nothing to avoid");
    let ids = [NATURAL, PERSONA_TRAIT, YES, NO];
    let expected = ["natural", "persona_trait", "自然", "自然でない"];
    assert_eq!(ids, expected, "the names of the questions and options");
}
