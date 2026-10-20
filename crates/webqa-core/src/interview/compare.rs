//! Compares this run's ledger with the previous one by issue number.

mod reach;

use std::collections::HashSet;

use crate::interview::issue::{Count, Issue, Place, Run};
use crate::interview::ledger::{Compared, Entry, Goal, Head, Ledger};
use reach::reached;

/// Why a previous ledger cannot be compared with this run. Carries no
/// text of either ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Mismatch {
    /// The persona's names differ.
    #[error("前回の台帳のペルソナが今回と違う")]
    Persona,
    /// The written purposes differ, or one run inferred its purpose and
    /// the other did not.
    #[error("前回の台帳の目的が今回と違う")]
    Goal,
    /// Both runs inferred their purposes, from different start URLs.
    #[error("目的を推測させた前回の実行と開始の URL が違う")]
    Start,
}

/// `now` compared with `previous` by issue number.
///
/// # Errors
///
/// [`Mismatch`] when the persona or the purpose differs.
pub fn compare(now: Ledger, previous: Option<&Ledger>, run: &Run<'_>) -> Result<Ledger, Mismatch> {
    match previous {
        None => Ok(now),
        Some(previous) => {
            let head = &now.head;
            check_previous(&previous.head, &head.persona, &head.goal, &head.start)
                .map(|()| merge(now, previous, run))
        }
    }
}

/// `now` with its issues marked by `previous`, and the issues only
/// `previous` has added.
fn merge(mut now: Ledger, previous: &Ledger, run: &Run<'_>) -> Ledger {
    let mut seen: HashSet<_> = now.entries.iter().map(|it| it.issue.number).collect();
    for entry in &mut now.entries {
        carry(entry, previous);
    }
    let finished = now.head.finished;
    let gone = previous
        .entries
        .iter()
        .filter(|it| seen.insert(it.issue.number));
    now.entries.extend(gone.map(|it| left(it, run, finished)));
    now.arrange();
    now
}

/// Whether `previous` can be compared with a run of `persona`, `goal` and
/// `start`: the same persona, and the same purpose, or both inferred from
/// one start URL.
///
/// # Errors
///
/// [`Mismatch`] when the persona or the purpose differs.
pub fn check_previous(
    previous: &Head,
    persona: &str,
    goal: &Goal,
    start: &str,
) -> Result<(), Mismatch> {
    if previous.persona == persona {
        same_goal(&previous.goal, &previous.start, goal, start)
    } else {
        Err(Mismatch::Persona)
    }
}

/// Whether the purposes are the same, or both inferred from one start URL.
fn same_goal(before: &Goal, before_start: &str, now: &Goal, start: &str) -> Result<(), Mismatch> {
    match (now, before) {
        (Goal::Written(a), Goal::Written(b)) => (a == b).then_some(()).ok_or(Mismatch::Goal),
        (Goal::Inferred(_), Goal::Inferred(_)) => {
            (start == before_start).then_some(()).ok_or(Mismatch::Start)
        }
        _ => Err(Mismatch::Goal),
    }
}

/// Marks `entry` as remaining with the previous selection, or as new.
fn carry(entry: &mut Entry, previous: &Ledger) {
    let before = previous
        .entries
        .iter()
        .find(|it| it.issue.number == entry.issue.number);
    entry.selected = before.is_some_and(|it| it.selected);
    entry.compared = before.map_or(Compared::New, |_| Compared::Remaining);
}

/// The previous `entry`, whose issue is not in this run, as this run's
/// ledger records it.
fn left(entry: &Entry, run: &Run<'_>, finished: usize) -> Entry {
    let now = reached(&entry.issue.place, run);
    let compared = if now >= entry.issue.count.reached.max(1) {
        Compared::Resolved
    } else {
        Compared::Unverified
    };
    let finished = match entry.issue.place {
        Place::Screen { .. } => finished,
        Place::Pattern(_) => now,
    };
    Entry {
        selected: entry.selected && compared == Compared::Unverified,
        compared,
        issue: Issue {
            shot: None,
            count: Count {
                occurred: 0,
                finished,
                reached: now,
            },
            ..entry.issue.clone()
        },
        ..entry.clone()
    }
}

#[cfg(test)]
mod tests;
