//! The state machine searched through with small constants: every state
//! reached from the start, by every fitting fact at every later time.
//!
//! A failure shows the facts and times passed from the start.

#[path = "exhaustive/absorb.rs"]
mod absorb;
#[path = "exhaustive/candidates.rs"]
mod candidates;
#[path = "exhaustive/exits.rs"]
mod exits;
#[path = "exhaustive/guards.rs"]
mod guards;
#[path = "exhaustive/limits.rs"]
mod limits;
#[path = "exhaustive/memory.rs"]
mod memory;
#[path = "exhaustive/report.rs"]
mod report;
#[path = "exhaustive/rows.rs"]
mod rows;
#[path = "exhaustive/view.rs"]
mod view;
#[path = "exhaustive/walk.rs"]
mod walk;

use webqa_core::machine::step;

use report::{Trace, group, traced};
use walk::{MAX, knows, report, started};

/// No failures.
const NONE: Vec<(&str, Trace)> = Vec::new();

#[test]
fn finished_is_absorbing() {
    let report = report();
    assert_eq!(report.broken_ended(), NONE, "ended runs stay ended");
    assert_ne!(report.found.absorbed, 0, "some run ended");
}

#[test]
fn budget_and_timeout_tie_yields_one_reason() {
    let report = report();
    assert_eq!(report.broken(group::TIE), NONE, "out of budget and of time");
    let ties = report.found.ties;
    assert_ne!(ties, 0, "some run ended out of budget at the time limit");
}

#[test]
fn forbidden_and_low_confidence_back_never_act() {
    assert_eq!(report().broken(group::ACTING), NONE, "acting");
}

#[test]
fn no_question_while_frozen_outside_or_on_paywall() {
    assert_eq!(report().broken(group::ASKING), NONE, "asking");
}

#[test]
fn budgets_and_window_stay_within_caps() {
    assert_eq!(report().broken(group::LIMITS), NONE, "limits");
}

#[test]
fn exit_reasons_are_backed_by_state() {
    assert_eq!(report().broken(group::REASONS), NONE, "reasons");
    assert_eq!(report().broken(group::GOING_ON), NONE, "going on");
}

#[test]
fn memory_and_notes_only_grow() {
    assert_eq!(report().broken(group::MEMORY), NONE, "memory");
}

#[test]
fn exploration_covers_every_row() {
    let report = report();
    assert_eq!(report.missed(), Vec::<&str>::new(), "every row is reached");
    let nodes = report.found.parents.len();
    assert!((1000..=MAX).contains(&nodes), "{nodes} states");
    // The facts and times that lead to the last state found reach it again.
    let path = report.found.parents.last().copied().flatten().unwrap();
    let (_, trace) = traced((&("the last state", path), report));
    let at = trace.last().map(|(_, at)| *at).unwrap();
    let top = report.candidates.pages.first().unwrap();
    let replayed = trace.into_iter().fold(started(top), |s, (facts, at)| {
        step(s.state().clone(), facts, at)
    });
    assert!(knows(report, &replayed, at), "the replayed state was found");
}
