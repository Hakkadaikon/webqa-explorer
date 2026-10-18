//! Tests of the questions and answers in exploration, where the judge may
//! not choose `done`, and on a screen without elements.

use std::collections::BTreeMap;

use super::ask::tests::ALL_OPS;
use super::{Answers, Ask, QuestionId, Unusable, Usage, questions, read_answers};
use crate::config::raw::Op;

/// The kinds of exploration: every kind but `done`, in the table's order.
const EXPLORE: [Op; 7] = [
    Op::Click,
    Op::Input,
    Op::ScrollDown,
    Op::ScrollUp,
    Op::Back,
    Op::Wait,
    Op::Stuck,
];

/// A step of exploration on a screen of two elements.
const EXPLORING: Ask<'static> = Ask {
    ops: &EXPLORE,
    elements: 2,
    goal: None,
};

/// A step of a scenario's stage, where `done` is among the kinds.
const SCENARIO: Ask<'static> = Ask {
    ops: &ALL_OPS,
    ..EXPLORING
};

/// A step on a screen without elements, with all eight kinds.
const EMPTY: Ask<'static> = Ask {
    ops: &ALL_OPS,
    elements: 0,
    goal: None,
};

type Row<'a> = (QuestionId, &'a [(&'a str, f64)]);

/// The answers of `rows`, with no error shown.
fn answers(rows: &[Row<'_>]) -> Answers {
    let error: Row<'_> = (QuestionId::error_shown(), &[("true", 0.0)]);
    let table = rows.iter().chain([&error]).map(|(id, options)| {
        let options = options.iter().map(|&(o, p)| (o.to_owned(), p)).collect();
        (id.clone(), options)
    });
    Answers {
        table: table.collect::<BTreeMap<_, _>>(),
        usage: Usage::default(),
    }
}

/// The operation's answer `options`, with the element `"1"` chosen.
fn chose(options: &[(&str, f64)]) -> Answers {
    answers(&[
        (QuestionId::operation(), options),
        (QuestionId::target(), &[("1", 0.9)]),
    ])
}

/// The operation and the elements read from `answers`.
fn read(answers: &Answers, ask: &Ask<'_>) -> Result<(Op, Vec<usize>), Unusable> {
    read_answers(answers, ask).map(|v| (v.op, v.targets))
}

/// The options of the question `id`, read from its serde form, if it was asked.
fn options(ask: &Ask<'_>, id: &QuestionId) -> Option<Vec<String>> {
    let questions = serde_json::to_value(questions(ask)).unwrap();
    let mut all = questions.as_array().unwrap().iter();
    let asked = all.find(|q| q["id"] == id.as_str()).cloned();
    asked.and_then(|q| serde_json::from_value(q["options"].clone()).ok())
}

#[test]
fn explore_mode_omits_done_option() {
    let want = [
        "click",
        "input",
        "scroll_down",
        "scroll_up",
        "back",
        "wait",
        "stuck",
    ]
    .map(str::to_owned);
    assert_eq!(
        options(&EXPLORING, &QuestionId::operation()),
        Some(want.to_vec()),
        "every kind but done"
    );
    let ids: Vec<QuestionId> = questions(&EXPLORING)
        .iter()
        .map(|q| q.id().clone())
        .collect();
    assert_eq!(
        ids,
        [
            QuestionId::operation(),
            QuestionId::target(),
            QuestionId::error_shown()
        ],
        "no goal to ask about"
    );
}

#[test]
fn done_in_explore_mode_is_anomaly() {
    let not_an_option = Err(Unusable::NotAnOption(QuestionId::operation()));
    assert_eq!(
        read(&chose(&[("done", 0.9)]), &EXPLORING),
        not_an_option,
        "done first"
    );
    assert_eq!(
        read(&chose(&[("click", 0.9), ("done", 0.1)]), &EXPLORING),
        not_an_option,
        "done after the chosen kind"
    );
    assert_eq!(
        read(&chose(&[("done", 0.9)]), &SCENARIO),
        Ok((Op::Done, Vec::new())),
        "done among the kinds of a scenario"
    );
}

#[test]
fn empty_screen_allows_only_untargeted_ops() {
    assert_eq!(
        options(&EMPTY, &QuestionId::target()),
        Some(Vec::new()),
        "no element to choose"
    );
    let scroll = answers(&[(QuestionId::operation(), &[("scroll_down", 0.9)])]);
    assert_eq!(
        read(&scroll, &EMPTY),
        Ok((Op::ScrollDown, Vec::new())),
        "scrolling needs no element"
    );
    let click = answers(&[(QuestionId::operation(), &[("click", 0.9)])]);
    assert_eq!(
        read(&click, &EMPTY),
        Err(Unusable::Missing(QuestionId::target())),
        "a click without an element"
    );
    assert_eq!(
        read(&chose(&[("click", 0.9)]), &EMPTY),
        Err(Unusable::NotAnOption(QuestionId::target())),
        "a click on an element not on the screen"
    );
}
