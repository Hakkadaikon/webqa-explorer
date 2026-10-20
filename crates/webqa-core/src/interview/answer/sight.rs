//! Reading the elements the persona found hard to see.

use crate::interview::ask::HARD_TO_SEE;
use crate::judge::{Answers, MAX_OPTIONS, QuestionId, Unusable};

/// The most elements kept from an answer; the ones listed after are cut.
pub const MAX_HARD_TO_SEE: usize = 3;

/// What may lead a line in front of its number.
const BULLETS: [char; 4] = ['-', '*', '・', '•'];

/// The digits, each as the half-width and the full-width one.
const DIGITS: [&str; 10] = [
    "0０", "1１", "2２", "3３", "4４", "5５", "6６", "7７", "8８", "9９",
];

/// An element the persona found hard to see.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardToSee {
    /// The element's number in the screen text, from 1.
    pub element: usize,
    /// Why, as the judge wrote it, trimmed.
    pub reason: String,
}

/// Reads the answer to `hard_to_see` among `elements` numbered elements.
/// A line that cannot be read as `number: reason` is dropped, and so is one
/// without a reason; a number listed again counts once; at most
/// [`MAX_HARD_TO_SEE`] are kept.
///
/// # Errors
///
/// [`Unusable::Missing`] when there is no answer, and
/// [`Unusable::NotAnOption`] for a number that was not asked about.
pub fn read(answers: &Answers, elements: usize) -> Result<Vec<HardToSee>, Unusable> {
    let id = QuestionId::new(HARD_TO_SEE);
    let text = answers
        .text(&id)
        .ok_or_else(|| Unusable::Missing(id.clone()))?;
    listed(text, elements.min(MAX_OPTIONS)).ok_or(Unusable::NotAnOption(id))
}

/// The elements of the lines of `text`, or `None` at the first number not
/// asked about.
fn listed(text: &str, asked: usize) -> Option<Vec<HardToSee>> {
    let mut listed: Vec<HardToSee> = Vec::new();
    for (element, reason) in text.lines().filter_map(numbered) {
        if !(1..=asked).contains(&element) {
            return None;
        }
        if listed.iter().all(|seen| seen.element != element) {
            listed.push(HardToSee {
                element,
                reason: reason.to_owned(),
            });
        }
    }
    listed.truncate(MAX_HARD_TO_SEE);
    Some(listed)
}

/// The number and the reason of a `number: reason` line, with the bullet
/// and the full-width forms allowed; `None` for any other line.
fn numbered(line: &str) -> Option<(usize, &str)> {
    let line = line.trim().trim_start_matches(BULLETS);
    let (number, reason) = line.split_once([':', '：'])?;
    let reason = reason.trim();
    (!reason.is_empty()).then_some((value(number.trim())?, reason))
}

/// The number written in half-width or full-width digits. One too large
/// for `usize` stops at the largest, which no element has.
fn value(number: &str) -> Option<usize> {
    let value = number.chars().try_fold(0_usize, |value, c| {
        Some(value.saturating_mul(10).saturating_add(digit(c)?))
    });
    value.filter(|_| !number.is_empty())
}

fn digit(c: char) -> Option<usize> {
    DIGITS.iter().position(|d| d.contains(c))
}
