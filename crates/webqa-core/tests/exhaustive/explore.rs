//! Explore mode: no `done` to choose, the moves when no element is left,
//! and the goal only after a question.

use webqa_core::effect::{Effect, Outside};
use webqa_core::judge::{Question, QuestionId};
use webqa_core::machine::ExitReason;

use super::candidates::pages::{CART, EMPTY};
use super::rows::{Row, not_an_option, row};
use super::view::{Property, Transition, all, implies};

/// The goal of explore mode, and what is offered in it.
pub static EXPLORE: [Property; 2] = [
    Property {
        name: "the goal of explore mode comes after a question",
        holds: |t| implies(achieved(t), t.after().budget().used() >= 1),
    },
    Property {
        name: "done is never offered in explore mode",
        holds: |t| !operations(t).iter().any(|op| op == "done"),
    },
];

fn achieved(t: &Transition) -> bool {
    t.reason() == Some(ExitReason::GoalAchieved)
}

/// The operation kinds the transition asks the judge to choose from.
pub fn operations(t: &Transition) -> Vec<String> {
    let questions = match t.step.effect() {
        Some(Effect::Outside(Outside::Ask(asked))) => asked.questions.as_slice(),
        Some(Effect::Outside(Outside::Sleep(_)) | Effect::Page(_)) | None => &[],
    };
    let operation = QuestionId::operation();
    questions
        .iter()
        .filter(|q| *q.id() == operation)
        .flat_map(choices)
        .collect()
}

fn choices(question: &Question) -> Vec<String> {
    match question {
        Question::Choice { options, .. } => options.clone(),
        Question::Noul { .. } | Question::Score { .. } | Question::Text { .. } => Vec::new(),
    }
}

/// What the search in explore mode must reach.
pub fn explored_rows() -> Vec<Row> {
    let done = not_an_option(QuestionId::operation());
    vec![
        row("the goal from the empty page", |t| {
            all(&[t.root == EMPTY, achieved(t)])
        }),
        row("the goal from the page with only a forbidden button", |t| {
            all(&[t.root == CART, achieved(t)])
        }),
        row("only going back and scrolling offered", |t| {
            operations(t) == ["scroll_down", "scroll_up", "back"]
        }),
        row("done answered is not an option", move |t| {
            all(&[t.op == Some("done"), t.adds(&done)])
        }),
    ]
}
