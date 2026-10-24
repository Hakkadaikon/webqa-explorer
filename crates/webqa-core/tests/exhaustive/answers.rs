//! The replies the judge may give.

use std::collections::BTreeMap;

use webqa_core::judge::{Answers, JudgeError, QuestionId, Usage};
use webqa_core::machine::{Facts, Heard};

use super::Choice;

/// The operation kinds answered. `drag` is not offered.
const OPS: [&str; 7] = [
    "click",
    "back",
    "scroll_down",
    "enter",
    "done",
    "stuck",
    "drag",
];

/// A sure and an unsure probability of the operation kind.
const CONFIDENCES: [f64; 2] = [0.9, 0.5];

/// The element numbers answered. `99` is on no page.
const TARGETS: [&str; 3] = ["1", "3", "99"];

/// The answers choosing `op` with `p` and the element `target`, seeing
/// neither an error nor the goal reached.
fn answers(op: &str, p: f64, target: &str) -> Answers {
    let no = || vec![("true".to_owned(), 0.0)];
    Answers {
        table: BTreeMap::from([
            (QuestionId::operation(), vec![(op.to_owned(), p)]),
            (QuestionId::target(), vec![(target.to_owned(), 1.0)]),
            (QuestionId::error_shown(), no()),
            (QuestionId::goal_reached(), no()),
        ]),
        usage: Usage::default(),
    }
}

/// The reply with the answers, and the operation kind in it.
fn reply((op, p, target): (&'static str, f64, &str)) -> Choice {
    Choice {
        facts: Facts::Outside(Heard::Answered(Ok(answers(op, p, target)))),
        op: Some(op),
    }
}

/// Every combination of operation kind, probability and element.
fn combinations() -> Vec<(&'static str, f64, &'static str)> {
    let pairs = OPS
        .iter()
        .flat_map(|op| CONFIDENCES.iter().map(move |p| (*op, *p)));
    pairs
        .flat_map(|(op, p)| TARGETS.iter().map(move |t| (op, p, *t)))
        .collect()
}

/// The judge failing with `error`.
const fn failed(error: JudgeError) -> Choice {
    Choice {
        facts: Facts::Outside(Heard::Answered(Err(error))),
        op: None,
    }
}

/// The 42 replies with answers, then the judge out of answers, then the
/// judge out of reach.
pub fn replies() -> Vec<Choice> {
    let failures = [JudgeError::Exhausted, JudgeError::Api].map(failed);
    combinations()
        .into_iter()
        .map(reply)
        .chain(failures)
        .collect()
}
