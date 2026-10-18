//! Tests of the provisional step.

use std::collections::BTreeMap;

use super::*;
use crate::judge::Usage;
use crate::redact::Redactor;
use crate::screen::{Element, Rect};

const VIEWPORT: Viewport = Viewport {
    width: 100,
    height: 100,
};

fn button(name: &str, y: i64) -> Element {
    Element {
        role: "button".to_owned(),
        name: name.to_owned(),
        value: None,
        rect: Some(Rect {
            x: 0,
            y,
            width: 10,
            height: 10,
        }),
    }
}

/// "隠れ" is below the viewport, so the text numbers "ヘルプ" 1 and "保存" 2.
fn seen() -> Seen {
    let screen = Screen {
        url: url::Url::parse("https://app.example.test/").unwrap(),
        elements: vec![button("隠れ", 500), button("ヘルプ", 0), button("保存", 20)],
    };
    Seen {
        screen: Redactor::default().screen(screen),
        viewport: VIEWPORT,
    }
}

fn ops() -> Vec<String> {
    vec!["click".to_owned(), "back".to_owned()]
}

fn state() -> State {
    State {
        last: Box::new(seen()),
        ops: ops(),
    }
}

fn button_id(name: &str) -> ElementId {
    ElementId {
        role: "button".to_owned(),
        name: name.to_owned(),
        nth: 0,
    }
}

fn table(options: &[(&str, f64)]) -> Vec<(String, f64)> {
    options.iter().map(|(o, p)| ((*o).to_owned(), *p)).collect()
}

/// The judge chose among `ops` and chose `number`, over "1"; an empty `ops`
/// leaves the operation without options.
fn answered(ops: &[(&str, f64)], number: &str) -> Step {
    let answers = Answers {
        table: BTreeMap::from([
            (QuestionId::operation(), table(ops)),
            (QuestionId::target(), table(&[(number, 0.6), ("1", 0.3)])),
        ]),
        usage: Usage::default(),
    };
    step(state(), Facts::Outside(Heard::Answered(Ok(answers))))
}

fn touched(op: &str, target: Option<ElementId>, confidence: f64) -> Step {
    let touch = Touch {
        op: op.to_owned(),
        target,
        confidence,
    };
    Step::Continue(state(), Effect::Page(PageEffect::Touch(touch)))
}

fn asking() -> Step {
    let questions = Questions {
        screen: screen_text(&seen().screen, VIEWPORT),
        questions: step_questions(&ops(), 2),
    };
    Step::Continue(state(), Effect::Outside(Outside::Ask(questions)))
}

#[test]
fn start_asks_about_the_visible_elements() {
    assert_eq!(
        start(seen(), ops()),
        asking(),
        "asks about the two visible elements"
    );
    assert_eq!(
        screen_text(&seen().screen, VIEWPORT).as_inner().as_str(),
        "1: button \"ヘルプ\"\n2: button \"保存\"",
        "the text numbers the visible elements only"
    );
}

#[test]
fn answers_touch_the_most_probable_operation_and_element() {
    assert_eq!(
        answered(&[("back", 0.05), ("click", 0.9), ("wait", 0.05)], "2"),
        touched("click", Some(button_id("保存")), 0.9),
        "the second visible element is 保存"
    );
    assert_eq!(
        answered(&[("click", 0.8), ("back", 0.2)], "1"),
        touched("click", Some(button_id("ヘルプ")), 0.8),
        "the hidden element is not numbered"
    );
}

#[test]
fn element_numbers_outside_the_text_touch_no_element() {
    for number in ["0", "3", "x", ""] {
        assert_eq!(
            answered(&[("back", 0.9)], number),
            touched("back", None, 0.9),
            "no element has the number {number:?}"
        );
    }
}

#[test]
fn no_operation_ends_unanswered() {
    assert_eq!(
        answered(&[], "1"),
        Step::Finished(Outcome::Unanswered),
        "nothing to touch without an operation"
    );
}

#[test]
fn judge_failure_ends_the_run() {
    let failed = Heard::Answered(Err(JudgeError::Exhausted));
    assert_eq!(
        step(state(), Facts::Outside(failed)),
        Step::Finished(Outcome::JudgeFailed(JudgeError::Exhausted)),
        "the error is kept"
    );
}

#[test]
fn waking_asks_again() {
    assert_eq!(
        step(state(), Facts::Outside(Heard::Woke)),
        asking(),
        "the same questions as at the start"
    );
}

#[test]
fn observing_after_the_touch_ends_the_step() {
    assert_eq!(
        step(state(), Facts::Page(seen())),
        Step::Finished(Outcome::Stepped),
        "one step only"
    );
}

#[test]
fn only_a_finished_step_has_an_outcome() {
    assert_eq!(asking().into_outcome(), None, "still running");
    assert_eq!(
        Step::Finished(Outcome::Stepped).into_outcome(),
        Some(Outcome::Stepped),
        "the outcome it finished with"
    );
    assert_eq!(state().last(), &seen(), "the state keeps the observation");
}
