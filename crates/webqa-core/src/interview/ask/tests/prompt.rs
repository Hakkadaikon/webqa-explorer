//! Tests of the prompts of the questions.

use super::*;
use crate::config::persona::tests::persona;
use crate::config::purpose::tests::PURPOSE;
use crate::config::purpose::{INFERRED_GOAL, parse};

#[test]
fn every_kind_of_prompt_is_led() {
    let mut score = Question::Score {
        id: QuestionId::new("score"),
        prompt: "問い".to_owned(),
        levels: Vec::new(),
    };
    lead_with(&mut score, "前置き");
    assert_eq!(prompt(&score), "前置き\n\n問い", "a score too");
}

#[test]
fn prompt_includes_persona_fields() {
    assert_eq!(preamble(&persona(), GOAL), P, "every field of the persona");
    for question in questions(&step(Some(&shot()))) {
        let lead = format!("{P}\n\n");
        assert!(prompt(&question).starts_with(&lead), "{question:?}");
    }
    let block = Persona {
        name: "  佐藤さん  ".to_owned(),
        profile: "60代。\n週末に買い物をする\n".to_owned(),
        ..persona()
    };
    let preamble_lines = preamble(&block, GOAL);
    assert_eq!(
        lines(&preamble_lines).get(1..5),
        Some(
            &[
                "名前: 佐藤さん",
                "どんな人か: 60代。",
                "週末に買い物をする",
                "ITの慣れ: スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている",
            ][..]
        ),
        "trimmed, inner line breaks kept"
    );
    let none = Persona {
        avoids: Vec::new(),
        ..persona()
    };
    let text = preamble(&none, GOAL);
    assert!(
        text.contains("しないこと:\n目指すこと: 注文を確定できる画面まで進む"),
        "{text}"
    );
    let inferred = preamble(&persona(), INFERRED_GOAL);
    let expected = "目指すこと: 画面から、この画面で利用者にしてほしい行動を推測し、ペルソナとしてそれに向かう";
    assert!(inferred.contains(expected), "{inferred}");
}

/// The eight questions of a step with an image.
fn eight() -> Vec<Question> {
    vec![
        operation(owned(&["click", "back", "done"])),
        target(owned(&["1", "2"])),
        Question::Noul {
            id: QuestionId::error_shown(),
            prompt: p("画面にエラーの表示があるか"),
        },
        Question::Noul {
            id: QuestionId::goal_reached(),
            prompt: p("次のゴールを達成したか: 注文を確定できる画面まで進む"),
        },
        Question::Noul {
            id: QuestionId::new(HESITATION),
            prompt: p("次に何をするかで迷ったか"),
        },
        Question::Text {
            id: QuestionId::new(EXPECTATION),
            prompt: p("選んだ操作で何が起きると期待したかを、1文で書く"),
        },
        Question::Text {
            id: QuestionId::new(HARD_TO_SEE),
            prompt: p(
                "スクリーンショットを見て、見えにくい、目立たない、読みにくいと感じた要素を、1行に1つ「番号: 理由」の形で書く。番号は画面の一覧の番号。無ければ何も書かない",
            ),
        },
        Question::Text {
            id: QuestionId::new(PURPOSE_GUESS),
            prompt: p("この画面は、利用者に何をしてもらうための画面だと思うかを、1文で書く"),
        },
    ]
}

#[test]
fn one_request_asks_eight_items() {
    let shot = shot();
    let qs = questions(&step(Some(&shot)));
    assert_eq!(qs, eight(), "the four of every mode, then the four added");
    let mut judge = Recorder::new(true);
    sent(&mut judge, &cart_text(), Some(&shot), &qs);
    let asked: Vec<_> = judge.calls.into_iter().map(|call| call.questions).collect();
    assert_eq!(asked, vec![eight()], "one call with the eight questions");
}

#[test]
fn asks_purpose_guess_even_when_written() {
    let written = parse(PURPOSE).unwrap().stage().goal;
    let shot = shot();
    for goal in [written.as_str(), INFERRED_GOAL] {
        for image in [Some(&shot), None] {
            let step = Step {
                goal,
                ..step(image)
            };
            let qs = questions(&step);
            let last = qs.last().map(Question::id);
            assert_eq!(last, Some(&QuestionId::new(PURPOSE_GUESS)), "{goal}");
        }
    }
}

#[test]
fn target_options_are_all_viewport_elements() {
    let page = screen(vec![
        element("link", "カート", Some(IN)),
        element("link", "ヘルプ", Some(IN)),
        element("link", "フッター", Some(OUT)),
        element("button", "矩形なし", None),
    ]);
    let redacted = Redactor::default().screen(page);
    let count = elements(&redacted, VIEWPORT);
    assert_eq!(count, 2, "the elements inside the viewport");
    let text = screen_text(&redacted, VIEWPORT);
    let expected = "1: link \"カート\"\n2: link \"ヘルプ\"";
    assert_eq!(text.as_inner().as_str(), expected, "the same numbers");
    let all: Vec<String> = (1..=255).map(|n| n.to_string()).collect();
    let cases = [(count, owned(&["1", "2"])), (0, Vec::new()), (300, all)];
    for (elements, want) in cases {
        assert_eq!(offered(elements), Some(target(want)), "{elements}");
    }
}

/// The question of the element when the screen text has `elements` lines.
fn offered(elements: usize) -> Option<Question> {
    let step = Step {
        elements,
        ..step(None)
    };
    questions(&step).get(1).cloned()
}

/// The operation kinds of the settings, with the options offered.
const DONE_OFFERED: [(&[Op], &[&str]); 5] = [
    (&[Op::Click, Op::Back], &["click", "back", "done"]),
    (&[], &["done"]),
    (&[Op::Done, Op::Click], &["done", "click"]),
    (&[Op::Click, Op::Done, Op::Done], &["click", "done"]),
    (
        &[
            Op::Click,
            Op::Input,
            Op::ScrollDown,
            Op::ScrollUp,
            Op::Back,
            Op::Wait,
            Op::Done,
            Op::Stuck,
        ],
        &[
            "click",
            "input",
            "scroll_down",
            "scroll_up",
            "back",
            "wait",
            "done",
            "stuck",
        ],
    ),
];

#[test]
fn interview_offers_done_option() {
    for (ops, want) in DONE_OFFERED {
        let step = Step { ops, ..step(None) };
        let got = questions(&step).first().cloned();
        assert_eq!(got, Some(operation(owned(want))), "{ops:?}");
    }
    assert_eq!(
        with_done(&[Op::Click, Op::Back]),
        [Op::Click, Op::Back, Op::Done],
        "done added at the end"
    );
    assert_eq!(with_done(&[]), [Op::Done], "done alone");
    assert_eq!(
        with_done(&[Op::Done, Op::Click]),
        [Op::Done, Op::Click],
        "done kept where it is"
    );
}
