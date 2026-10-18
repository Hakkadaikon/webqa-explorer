//! Reading the answers of one step.

use std::cmp::Ordering;

use super::{Answers, Ask, MAX_OPTIONS, QuestionId, Usage, op_name};
use crate::config::raw::Op;

/// What the answers of a step say, once checked.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    /// The chosen operation: the first of the ranked options.
    pub op: Op,
    /// Its probability. `None` when the answer had none (low confidence).
    pub confidence: Option<f64>,
    /// Element numbers (from 1) ranked by probability. Empty when `op`
    /// needs no element.
    pub targets: Vec<usize>,
    /// Probability that the page shows an error. `None` when missing.
    pub error_shown: Option<f64>,
    /// Probability that the goal is reached. `None` when not asked or missing.
    pub goal_reached: Option<f64>,
    /// What the call used.
    pub usage: Usage,
}

/// Answers that arrived but cannot be used. Carries only the name of the
/// question, never what the judge returned.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Unusable {
    /// The question has no answer, or no option.
    #[error("判断モデルの答えに {} が無い", .0.as_str())]
    Missing(QuestionId),
    /// An option of the answer was not asked.
    #[error("判断モデルの答えの {} が問いの選択肢に無い", .0.as_str())]
    NotAnOption(QuestionId),
    /// A probability is not between 0 and 1.
    #[error("判断モデルの答えの {} の確率が 0 以上 1 以下でない", .0.as_str())]
    OutOfRange(QuestionId),
}

/// Checks the answers against what `ask` asked.
///
/// # Errors
///
/// The first problem found, checking the operation, the element, the error
/// and the goal in this order.
pub fn read_answers(answers: &Answers, ask: &Ask<'_>) -> Result<Verdict, Unusable> {
    let (op, confidence) = read_op(answers, ask.ops)?;
    let targets = read_targets(answers, op, ask.elements)?;
    let error_shown = read_flag(answers, &QuestionId::error_shown())?;
    let goal_reached = read_goal(answers, ask.goal)?;
    Ok(Verdict {
        op,
        confidence,
        targets,
        error_shown,
        goal_reached,
        usage: answers.usage,
    })
}

/// Operations that need an element.
const TARGETED: [Op; 2] = [Op::Click, Op::Input];

/// The first of the ranked operations, with its probability.
fn read_op(answers: &Answers, ops: &[Op]) -> Result<(Op, Option<f64>), Unusable> {
    let id = QuestionId::operation();
    let ranked = ranked(answers, &id, |name| {
        ops.iter().copied().find(|&op| op_name(op) == name)
    })?;
    ranked.into_iter().next().ok_or(Unusable::Missing(id))
}

/// The element numbers ranked, or none when `op` needs no element.
fn read_targets(answers: &Answers, op: Op, elements: usize) -> Result<Vec<usize>, Unusable> {
    if !TARGETED.contains(&op) {
        return Ok(Vec::new());
    }
    let asked = elements.min(MAX_OPTIONS);
    let ranked = ranked(answers, &QuestionId::target(), |name| number(name, asked))?;
    Ok(ranked.into_iter().map(first).collect())
}

/// The element number written exactly as `name`, among those asked.
fn number(name: &str, asked: usize) -> Option<usize> {
    (1..=asked).find(|n| n.to_string() == name)
}

fn first<T, U>((t, _): (T, U)) -> T {
    t
}

/// The probability of `"true"`, the only option of a yes-or-no answer.
fn read_flag(answers: &Answers, id: &QuestionId) -> Result<Option<f64>, Unusable> {
    let ranked = ranked(answers, id, |name| (name == "true").then_some(()))?;
    Ok(ranked.first().and_then(|&((), p)| p))
}

/// The goal's flag, read only when there is a goal.
fn read_goal(answers: &Answers, goal: Option<&str>) -> Result<Option<f64>, Unusable> {
    goal.map(|_| read_flag(answers, &QuestionId::goal_reached()))
        .transpose()
        .map(Option::flatten)
}

/// The options of `id` read by `option`, most probable first. Missing
/// probabilities come last; ties keep the order of the answer. An option
/// given again is not an option.
fn ranked<T>(
    answers: &Answers,
    id: &QuestionId,
    option: impl Fn(&str) -> Option<T>,
) -> Result<Vec<(T, Option<f64>)>, Unusable> {
    let options = options(answers, id)?;
    let mut ranked = options
        .iter()
        .enumerate()
        .map(|(i, (name, p))| {
            let read = option(name).filter(|_| first_given(options, name) == Some(i));
            read_option(id, read, *p)
        })
        .collect::<Result<Vec<_>, _>>()?;
    ranked.sort_by(likelier_first);
    Ok(ranked)
}

/// Where `name` is first given among the options.
fn first_given(options: &[(String, f64)], name: &str) -> Option<usize> {
    options.iter().position(|(given, _)| given == name)
}

/// The options of `id`, or [`Unusable::Missing`] when there are none.
fn options<'a>(answers: &'a Answers, id: &QuestionId) -> Result<&'a [(String, f64)], Unusable> {
    answers
        .table
        .get(id)
        .map(Vec::as_slice)
        .filter(|options| !options.is_empty())
        .ok_or_else(|| Unusable::Missing(id.clone()))
}

/// Orders by probability, highest first and missing last.
fn likelier_first<T>(a: &(T, Option<f64>), b: &(T, Option<f64>)) -> Ordering {
    rank(b.1).total_cmp(&rank(a.1))
}

fn read_option<T>(
    id: &QuestionId,
    option: Option<T>,
    p: f64,
) -> Result<(T, Option<f64>), Unusable> {
    let option = option.ok_or_else(|| Unusable::NotAnOption(id.clone()))?;
    Ok((option, probability(id, p)?))
}

/// `None` for a missing probability (NaN), and [`Unusable::OutOfRange`]
/// for anything else not from 0 to 1.
fn probability(id: &QuestionId, p: f64) -> Result<Option<f64>, Unusable> {
    if p.is_nan() {
        Ok(None)
    } else if (0.0..=1.0).contains(&p) {
        Ok(Some(p))
    } else {
        Err(Unusable::OutOfRange(id.clone()))
    }
}

/// Puts a missing probability below every probability.
fn rank(p: Option<f64>) -> f64 {
    p.unwrap_or(-1.0)
}

#[cfg(test)]
mod tests;
