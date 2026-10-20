//! Checking the answers of an interview step, and of the mismatch
//! questions.

mod mismatch;
mod sight;

pub use mismatch::{Match, read_mismatch};
pub use sight::HardToSee;

use super::ask::{EXPECTATION, HESITATION, PURPOSE_GUESS, Step, with_done};
use crate::judge::{Answers, QuestionId, Unusable, Verdict, read_answers};

/// What the answers of an interview step say, once checked.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    /// The four questions of every mode.
    pub verdict: Verdict,
    /// How hesitant the persona was, in `[0, 1]`.
    pub hesitation: f64,
    /// What the persona expected, as the judge wrote it.
    pub expectation: String,
    /// The elements hard to see. `None` when no image was sent, so the
    /// question was not asked.
    pub hard_to_see: Option<Vec<HardToSee>>,
    /// The purpose the persona guessed, as the judge wrote it.
    pub purpose_guess: String,
}

/// Checks the answers against what `step` asked.
///
/// # Errors
///
/// The first problem found: the four questions of every mode as
/// [`read_answers`] checks them, then hesitation, expectation, the
/// elements hard to see (only when the step has an image) and the purpose
/// guess, in this order.
pub fn read_step(answers: &Answers, step: &Step<'_>) -> Result<Reply, Unusable> {
    read_with(answers, step, read_expectation)
}

/// As [`read_step`], for a step that did not ask what was expected, since
/// no operation was carried out. The expectation of the reply is empty.
///
/// # Errors
///
/// As [`read_step`], without the expectation.
pub fn read_without_expectation(answers: &Answers, step: &Step<'_>) -> Result<Reply, Unusable> {
    read_with(answers, step, |_| Ok(String::new()))
}

/// How the expectation is read.
type Expectation = fn(&Answers) -> Result<String, Unusable>;

fn read_with(
    answers: &Answers,
    step: &Step<'_>,
    expectation: Expectation,
) -> Result<Reply, Unusable> {
    let ops = with_done(step.ops);
    let verdict = read_answers(answers, &step.ask(&ops))?;
    read_added(answers, step, verdict, expectation)
}

/// What the persona expected, as the judge wrote it.
///
/// # Errors
///
/// [`Unusable::Missing`] when the answer is missing, blank, or has more
/// than one text.
pub fn read_expectation(answers: &Answers) -> Result<String, Unusable> {
    read_text(answers, EXPECTATION)
}

/// Reads the four added questions, in the order they are asked.
fn read_added(
    answers: &Answers,
    step: &Step<'_>,
    verdict: Verdict,
    expectation: Expectation,
) -> Result<Reply, Unusable> {
    let hesitation = read_hesitation(answers)?;
    let expectation = expectation(answers)?;
    let hard_to_see = read_sight(answers, step)?;
    let purpose_guess = read_text(answers, PURPOSE_GUESS)?;
    Ok(Reply {
        verdict,
        hesitation,
        expectation,
        hard_to_see,
        purpose_guess,
    })
}

/// The probability of `"true"`, the one option of the hesitation answer.
fn read_hesitation(answers: &Answers) -> Result<f64, Unusable> {
    let id = QuestionId::new(HESITATION);
    let p = only_true(answers, &id)?;
    degree(id, p)
}

/// The probability of the answer, when its one option is `"true"`.
fn only_true(answers: &Answers, id: &QuestionId) -> Result<f64, Unusable> {
    match answers.table.get(id).map(Vec::as_slice) {
        None | Some([]) => Err(Unusable::Missing(id.clone())),
        Some([(name, p)]) if name == "true" => Ok(*p),
        Some(_) => Err(Unusable::NotAnOption(id.clone())),
    }
}

/// `p` as a degree from 0 to 1. NaN is no degree.
fn degree(id: QuestionId, p: f64) -> Result<f64, Unusable> {
    if p.is_nan() {
        Err(Unusable::Missing(id))
    } else if (0.0..=1.0).contains(&p) {
        Ok(p)
    } else {
        Err(Unusable::OutOfRange(id))
    }
}

/// The free text written for `name`, as written, unless it is blank.
fn read_text(answers: &Answers, name: &str) -> Result<String, Unusable> {
    let id = QuestionId::new(name);
    answers
        .text(&id)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_owned)
        .ok_or(Unusable::Missing(id))
}

/// The elements hard to see, read only when the step sent an image.
fn read_sight(answers: &Answers, step: &Step<'_>) -> Result<Option<Vec<HardToSee>>, Unusable> {
    step.image
        .map(|_| sight::read(answers, step.elements))
        .transpose()
}

#[cfg(test)]
mod tests;
