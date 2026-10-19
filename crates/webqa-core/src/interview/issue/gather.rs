//! Gathering the occurrences of the finished trials into issues.

use super::find::{At, Found, found};
use super::{Count, Issue, Key, Number, Place, Run, ScreenMark, Spot};
use crate::interview::outcome::{Ending, Outcome};
use crate::screen::{ScreenId, screen_id};

/// The hesitation, visibility and unreached issues of the finished trials
/// of `run`, in the order their keys were first seen. Empty when no trial
/// finished.
#[must_use]
pub fn step_issues(run: &Run<'_>) -> Vec<Issue> {
    let finished = finished(run);
    let reached: Vec<Vec<ScreenId>> = finished
        .iter()
        .map(|at| {
            at.record
                .steps
                .iter()
                .map(|seen| screen_id(seen.screen))
                .collect()
        })
        .collect();
    let groups = groups(finished.iter().flat_map(|&at| found(at)));
    groups
        .into_iter()
        .map(|group| issue(group, &reached))
        .collect()
}

/// The finished trials of `run`, with their numbers.
fn finished<'r>(run: &Run<'r>) -> Vec<At<'r>> {
    run.trials
        .iter()
        .zip(1..)
        .filter_map(|(record, trial)| {
            ending(record.outcome).map(|ending| At {
                record,
                trial,
                images: run.images,
                ending,
            })
        })
        .collect()
}

/// How a finished trial ended. `None` for a trial cut short.
const fn ending(outcome: Outcome) -> Option<Ending> {
    match outcome {
        Outcome::Finished(ending) => Some(ending),
        Outcome::Cut(_) => None,
    }
}

/// The occurrences of one key: the first, and in how many trials.
struct Group {
    first: Found,
    occurred: usize,
    last_trial: usize,
}

impl Group {
    const fn new(first: Found) -> Self {
        let last_trial = first.shot.trial;
        Self {
            first,
            occurred: 1,
            last_trial,
        }
    }

    /// Counts another occurrence in `trial`, once per trial.
    const fn note(&mut self, trial: usize) {
        if self.last_trial != trial {
            self.occurred += 1;
            self.last_trial = trial;
        }
    }
}

/// The occurrences grouped by key, in the order the keys were first seen.
fn groups(found: impl Iterator<Item = Found>) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    for occurrence in found {
        add(&mut groups, occurrence);
    }
    groups
}

fn add(groups: &mut Vec<Group>, occurrence: Found) {
    match groups.iter_mut().find(|g| g.first.same_key(&occurrence)) {
        Some(group) => group.note(occurrence.shot.trial),
        None => groups.push(Group::new(occurrence)),
    }
}

/// The issue of `group`, given the screens each finished trial reached.
fn issue(group: Group, reached: &[Vec<ScreenId>]) -> Issue {
    let first = group.first;
    let number = Number::of(Key {
        spot: Spot::Screen {
            screen: &first.screen,
            element: first.element.as_ref(),
        },
        kind: first.detail.kind(),
    });
    let count = Count {
        occurred: group.occurred,
        finished: reached.len(),
        reached: reached
            .iter()
            .filter(|ids| ids.contains(&first.screen))
            .count(),
    };
    Issue {
        number,
        place: Place::Screen {
            url: first.url,
            mark: ScreenMark::of(&first.screen),
        },
        target: first.target,
        detail: first.detail,
        count,
        shot: Some(first.shot),
        path: first.path,
        confidence: first.confidence,
    }
}
