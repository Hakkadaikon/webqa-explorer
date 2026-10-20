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
