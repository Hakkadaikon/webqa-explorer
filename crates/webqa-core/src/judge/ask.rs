//! The questions of one step.

use serde_json::Value;

use super::{Question, QuestionId, step_questions};
use crate::config::raw::Op;

/// What a step asks about. Built once and used both to build the questions
/// and to read the answers, so the two always agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ask<'a> {
    /// Operation kinds the judge may choose, in this order.
    /// The caller leaves out `done` in exploration.
    pub ops: &'a [Op],
    /// Number of elements in the screen text (the viewport's elements).
    pub elements: usize,
    /// The goal of the current stage. `None` in exploration.
    pub goal: Option<&'a str>,
}

/// Most options one Choice may carry (JEV's limit).
pub const MAX_OPTIONS: usize = 255;

/// The name of `op` sent to the judge, as serde writes it.
#[must_use]
pub fn op_name(op: Op) -> String {
    serde_json::to_value(op)
        .ok()
        .as_ref()
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// The questions of one step: the operation kind, the element number,
/// whether the page shows an error, and whether the goal is reached (only
/// when there is a goal).
#[must_use]
pub fn questions(ask: &Ask<'_>) -> Vec<Question> {
    let mut questions = step_questions(&op_names(ask.ops), ask.elements.min(MAX_OPTIONS));
    questions.push(Question::Noul {
        id: QuestionId::error_shown(),
        prompt: "画面にエラーの表示があるか".to_owned(),
    });
    questions.extend(ask.goal.map(goal_question));
    questions.retain(not_an_empty_target);
    questions
}

/// Whether `question` is sent.
///
/// All are but the question about the element when it has no element to
/// offer, since the judge refuses a choice of nothing. An operation kind that needs an element is still read as before: chosen
/// without an element to answer, it is an unusable answer.
#[must_use]
pub fn not_an_empty_target(question: &Question) -> bool {
    !matches!(question, Question::Choice { id, options, .. }
        if *id == QuestionId::target() && options.is_empty())
}

/// The names of `ops` in order, each once, so that no option is sent twice.
fn op_names(ops: &[Op]) -> Vec<String> {
    let mut names = Vec::new();
    for name in ops.iter().copied().map(op_name) {
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

fn goal_question(goal: &str) -> Question {
    Question::Noul {
        id: QuestionId::goal_reached(),
        prompt: format!("次のゴールを達成したか: {goal}"),
    }
}

#[cfg(test)]
pub mod tests;
