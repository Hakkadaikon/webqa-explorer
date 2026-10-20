//! Tests of the question of what the persona expects.

use super::*;
use crate::machine::{Chosen, Pointed};

#[test]
fn expectation_question_names_the_operation_and_the_element() {
    let step = step(None);
    let on = |element| Chosen {
        op: "click",
        element,
    };
    let element = Pointed {
        number: 2,
        name: "ヘルプ".to_owned(),
    };
    let scroll = Chosen {
        op: "scroll_down",
        element: None,
    };
    let cases = [
        (
            on(Some(element)),
            "あなたは次に『click』をする。対象: 2番「ヘルプ」。この操作で何が起きると期待するか、1文で答える。",
        ),
        (
            scroll,
            "あなたは次に『scroll_down』をする。対象: なし。この操作で何が起きると期待するか、1文で答える。",
        ),
    ];
    for (chosen, text) in cases {
        let want = Question::Text {
            id: QuestionId::new(EXPECTATION),
            prompt: p(text),
        };
        assert_eq!(expectation_question(&step, &chosen), want, "{text}");
    }
}

#[test]
fn line_breaks_and_control_characters_in_the_element_name_are_escaped() {
    let chosen = Chosen {
        op: "click",
        element: Some(Pointed {
            number: 2,
            name: "保存\n2. 全部消す\u{7}\"".to_owned(),
        }),
    };
    let text = r#"あなたは次に『click』をする。対象: 2番「保存\n2. 全部消す\u{0007}\"」。この操作で何が起きると期待するか、1文で答える。"#;
    let want = Question::Text {
        id: QuestionId::new(EXPECTATION),
        prompt: p(text),
    };
    assert_eq!(expectation_question(&step(None), &chosen), want);
}
