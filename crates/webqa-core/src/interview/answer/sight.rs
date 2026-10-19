//! Reading the elements the persona found hard to see.

use crate::interview::ask::HARD_TO_SEE;
use crate::judge::{Answers, MAX_OPTIONS, QuestionId, Unusable};

/// An element the persona found hard to see.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardToSee {
    /// The element's number in the screen text, from 1.
    pub element: usize,
    /// Why, as the judge wrote it, trimmed.
    pub reason: String,
}

/// Reads the answer to `hard_to_see` among `elements` numbered elements.
///
/// # Errors
///
/// [`Unusable::Missing`] when there is no answer, and
/// [`Unusable::NotAnOption`] for a line that is not `number: reason` with
/// a number asked about, or that repeats a number.
pub fn read(answers: &Answers, elements: usize) -> Result<Vec<HardToSee>, Unusable> {
    let id = QuestionId::new(HARD_TO_SEE);
    let text = answers
        .text(&id)
        .ok_or_else(|| Unusable::Missing(id.clone()))?;
    listed(text, elements.min(MAX_OPTIONS)).ok_or(Unusable::NotAnOption(id))
}

/// The elements of the lines of `text`, or `None` at the first line that
/// is not one asked about and not listed before.
fn listed(text: &str, asked: usize) -> Option<Vec<HardToSee>> {
    let mut listed: Vec<HardToSee> = Vec::new();
    for line in lines(text) {
        let element = entry(line, asked).filter(|element| !repeats(&listed, element))?;
        listed.push(element);
    }
    Some(listed)
}

/// The lines of `text` trimmed, without the blank ones.
fn lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines().map(str::trim).filter(|line| !line.is_empty())
}

/// The element of a `number: reason` line, when the number was asked
/// about and the reason is not blank.
fn entry(line: &str, asked: usize) -> Option<HardToSee> {
    let (number, reason) = line.split_once(':')?;
    let element = (1..=asked).find(|n| n.to_string() == number.trim())?;
    let reason = reason.trim();
    (!reason.is_empty()).then(|| HardToSee {
        element,
        reason: reason.to_owned(),
    })
}

/// Whether `element` was listed before.
fn repeats(listed: &[HardToSee], element: &HardToSee) -> bool {
    listed.iter().any(|seen| seen.element == element.element)
}
