//! The ruling on a mismatch question, and the mismatch issue.

use super::record::path;
use super::{
    Count, Detail, Issue, Key, Kind, Number, Place, Record, Run, Shot, Spot, UncheckedPurpose,
};
use crate::config::purpose::UrlPattern;
use crate::interview::answer::Match;
use crate::interview::ask::MismatchQuestion;

/// What a mismatch question came to.
#[derive(Debug, Clone, PartialEq)]
pub enum Ruling {
    /// The judge said the guesses differ: a mismatch issue.
    Mismatch(Box<Issue>),
    /// The judge said the guesses mean the written purpose.
    Same,
    /// The answer was unusable: the screen purpose goes to the unchecked
    /// section.
    Unchecked(UncheckedPurpose),
}

/// The ruling on `question` from its answer: `None` when the answer was
/// unusable or the judge failed.
#[must_use]
pub fn ruling(question: &MismatchQuestion<'_>, answer: Option<Match>, run: &Run<'_>) -> Ruling {
    match answer {
        Some(Match::Different) => Ruling::Mismatch(Box::new(issue(question, run))),
        Some(Match::Same) => Ruling::Same,
        None => Ruling::Unchecked(unchecked(question)),
    }
}

/// The unchecked entry of `question`, for an unusable answer or a question
/// over the cap.
#[must_use]
pub fn unchecked(question: &MismatchQuestion<'_>) -> UncheckedPurpose {
    UncheckedPurpose {
        pattern: question.screen.pattern.as_str().to_owned(),
        purpose: question.screen.purpose.clone(),
    }
}

/// A finished trial that reached the pattern, with the number of the
/// trial and the first step that did.
struct Hit<'r> {
    record: &'r Record<'r>,
    trial: usize,
    /// The step's position in the trial, from 1.
    number: usize,
    /// The step's number in the event log.
    step: u64,
}

/// The mismatch issue of `question`.
fn issue(question: &MismatchQuestion<'_>, run: &Run<'_>) -> Issue {
    let pattern = &question.screen.pattern;
    let hits: Vec<Hit<'_>> = run
        .trials
        .iter()
        .zip(1..)
        .filter(|(record, _)| record.outcome.finished())
        .filter_map(|(record, trial)| hit(pattern, record, trial))
        .collect();
    let reached = hits.len();
    let first = hits.first();
    Issue {
        number: Number::of(Key {
            spot: Spot::Pattern(pattern.as_str()),
            kind: Kind::Mismatch,
        }),
        place: Place::Pattern(pattern.as_str().to_owned()),
        target: None,
        detail: Detail::Mismatch {
            purpose: question.screen.purpose.clone(),
            guesses: question.guesses.clone(),
        },
        count: Count {
            occurred: reached,
            finished: reached,
            reached,
        },
        shot: first.map(|hit| Shot {
            trial: hit.trial,
            step: hit.step,
        }),
        path: first.map_or_else(Vec::new, |hit| path(hit.record.steps, hit.number)),
        confidence: None,
    }
}

/// The first step of `record` at a screen that `pattern` matches.
fn hit<'r>(pattern: &UrlPattern, record: &'r Record<'r>, trial: usize) -> Option<Hit<'r>> {
    record
        .steps
        .iter()
        .zip(1..)
        .find(|(seen, _)| pattern.matches(&seen.screen.as_inner().url))
        .map(|(seen, number)| Hit {
            record,
            trial,
            number,
            step: seen.step,
        })
}

#[cfg(test)]
mod tests;
