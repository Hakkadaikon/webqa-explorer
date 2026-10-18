//! Reading the answer to a mismatch question.

use crate::interview::ask::{DIFFERENT, PURPOSE_MATCH, SAME};
use crate::judge::{Answers, QuestionId, Unusable};

/// What the judge answered to a mismatch question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Match {
    /// The guesses mean the written purpose.
    Same,
    /// At least one guess means something else: a purpose mismatch.
    Different,
}

/// Reads the answer to a mismatch question.
///
/// # Errors
///
/// [`Unusable::Missing`] when there is no answer or no probability,
/// [`Unusable::NotAnOption`] for an option that is neither choice or is
/// given twice, and [`Unusable::OutOfRange`] for a probability outside
/// `[0, 1]`. The caller files the screen purpose as unchecked.
pub fn read_mismatch(answers: &Answers) -> Result<Match, Unusable> {
    let id = QuestionId::new(PURPOSE_MATCH);
    let options = options(answers, &id)?;
    named(options, &id)?;
    in_range(options, &id)?;
    let (best, _) = answers.best(&id).ok_or(Unusable::Missing(id))?;
    Ok(choice(best))
}

/// The options of the answer, or [`Unusable::Missing`] when there are none.
fn options<'a>(answers: &'a Answers, id: &QuestionId) -> Result<&'a [(String, f64)], Unusable> {
    answers
        .table
        .get(id)
        .map(Vec::as_slice)
        .filter(|options| !options.is_empty())
        .ok_or_else(|| Unusable::Missing(id.clone()))
}

/// Every option is one of the two choices, given once.
fn named(options: &[(String, f64)], id: &QuestionId) -> Result<(), Unusable> {
    let named = options
        .iter()
        .enumerate()
        .all(|(i, (name, _))| is_choice(name) && first_given(options, name) == Some(i));
    named
        .then_some(())
        .ok_or_else(|| Unusable::NotAnOption(id.clone()))
}

fn is_choice(name: &str) -> bool {
    [SAME, DIFFERENT].contains(&name)
}

/// Where `name` is first given among the options.
fn first_given(options: &[(String, f64)], name: &str) -> Option<usize> {
    options.iter().position(|(given, _)| given == name)
}

/// No probability is outside `[0, 1]`. NaN is no probability.
fn in_range(options: &[(String, f64)], id: &QuestionId) -> Result<(), Unusable> {
    let outside = options.iter().any(|&(_, p)| out_of_range(p));
    (!outside)
        .then_some(())
        .ok_or_else(|| Unusable::OutOfRange(id.clone()))
}

fn out_of_range(p: f64) -> bool {
    !(p.is_nan() || (0.0..=1.0).contains(&p))
}

/// The choice named `name`, one of the two choices.
fn choice(name: &str) -> Match {
    match name {
        SAME => Match::Same,
        _ => Match::Different,
    }
}

#[cfg(test)]
mod tests;
