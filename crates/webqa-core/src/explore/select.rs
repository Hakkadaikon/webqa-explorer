//! The candidates of the exploration mode, in tiers.

#[cfg(test)]
mod tests;

use super::graph::{Entry, Memory, ScreenKey};
use crate::redact::Redacted;
use crate::screen::{ElementId, Screen, Viewport, element_ids, in_viewport};

/// One candidate. `number` is the line number in `screen_text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// The number of the element among those inside the viewport, from 1.
    pub number: usize,
    /// The identity of the element.
    pub id: ElementId,
}

/// The highest tier that has candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Candidates {
    /// Untouched elements of the current screen, without forbidden ones.
    Untouched(Vec<Candidate>),
    /// Operated elements with an unexplored branch, without forbidden ones.
    Unfinished(Vec<Candidate>),
    /// Back, scroll down and scroll up.
    Moves,
}

/// The candidates on the screen `at`, which [`Memory::visit`] returned for `screen`.
///
/// Only elements inside the viewport count, since only they get a number.
/// Elements missing from the memory, and an unknown `at`, give no candidates.
#[must_use]
pub fn candidates(
    memory: &Memory,
    at: ScreenKey,
    screen: &Redacted<Screen>,
    viewport: Viewport,
) -> Candidates {
    let rows = rows(memory, at, screen, viewport);
    non_empty(pick(&rows, Entry::untouched))
        .map(Candidates::Untouched)
        .or_else(|| {
            non_empty(pick(&rows, |entry| unfinished(memory, at, entry)))
                .map(Candidates::Unfinished)
        })
        .unwrap_or(Candidates::Moves)
}

/// The viewport elements known to the memory, numbered as in `screen_text`.
fn rows<'m>(
    memory: &'m Memory,
    at: ScreenKey,
    screen: &Redacted<Screen>,
    viewport: Viewport,
) -> Vec<(Candidate, &'m Entry)> {
    screen
        .as_inner()
        .elements
        .iter()
        .zip(element_ids(screen))
        .filter(|(element, _)| in_viewport(element, viewport))
        .zip(1..)
        .filter_map(|((_, id), number)| known(memory, at, Candidate { number, id }))
        .collect()
}

fn known(memory: &Memory, at: ScreenKey, candidate: Candidate) -> Option<(Candidate, &Entry)> {
    let entry = memory.entry(at, &candidate.id)?;
    Some((candidate, entry))
}

fn pick(rows: &[(Candidate, &Entry)], keep: impl Fn(&Entry) -> bool) -> Vec<Candidate> {
    rows.iter()
        .filter(|(_, entry)| keep(entry))
        .map(|(candidate, _)| candidate.clone())
        .collect()
}

/// Operated, not forbidden, and not explored yet.
fn unfinished(memory: &Memory, at: ScreenKey, entry: &Entry) -> bool {
    entry.operated && !entry.forbidden && !memory.finished(at, &entry.id)
}

fn non_empty(candidates: Vec<Candidate>) -> Option<Vec<Candidate>> {
    (!candidates.is_empty()).then_some(candidates)
}
