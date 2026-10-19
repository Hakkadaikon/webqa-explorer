//! The replies the judge may give.

use std::collections::BTreeMap;
use std::iter;

use webqa_core::judge::{Answers, JudgeError, QuestionId, Usage};
use webqa_core::machine::{Facts, Heard};

use super::Choice;

/// The operation kinds answered. `drag` is not offered.
const OPS: [&str; 6] = ["click", "back", "scroll_down", "done", "stuck", "drag"];

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

/// The 36 replies with answers, then the judge failing.
pub fn replies() -> Vec<Choice> {
    let failed = Choice {
        facts: Facts::Outside(Heard::Answered(Err(JudgeError::Exhausted))),
        op: None,
    };
    combinations()
        .into_iter()
        .map(reply)
        .chain(iter::once(failed))
        .collect()
}
