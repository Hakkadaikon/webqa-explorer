//! The state machine searched through with small constants: from the
//! start pages, every fitting fact at every later time is passed to every
//! state reached, telling states apart by a key of what decides the next
//! steps. Three searches: a scenario without stages, a scenario with
//! stages, and explore mode.
//!
//! A failure shows how many transitions broke the property, and the facts
//! and times passed from the start to the first few.

#[path = "exhaustive/absorb.rs"]
mod absorb;
#[path = "exhaustive/candidates.rs"]
mod candidates;
#[path = "exhaustive/exits.rs"]
mod exits;
#[path = "exhaustive/explore.rs"]
mod explore;
#[path = "exhaustive/guards.rs"]
mod guards;
#[path = "exhaustive/key.rs"]
mod key;
#[path = "exhaustive/limits.rs"]
mod limits;
#[path = "exhaustive/memory.rs"]
mod memory;
#[path = "exhaustive/priority.rs"]
mod priority;
#[path = "exhaustive/report.rs"]
mod report;
#[path = "exhaustive/rows.rs"]
mod rows;
#[path = "exhaustive/view.rs"]
mod view;
#[path = "exhaustive/walk.rs"]
mod walk;
#[path = "exhaustive/walks.rs"]
mod walks;

use webqa_core::machine::{ExitReason, step};

use key::knows;
use report::{Broken, group, traced};
use walk::MAX;
use walks::{explored, report, staged, started, walks};

/// No failures.
const NONE: Broken = (0, Vec::new());

/// The groups that hold in every search.
const SHARED: [usize; 7] = [
    group::ACTING,
    group::ASKING,
    group::LIMITS,
    group::REASONS,
    group::GOING_ON,
    group::TIE,
    group::MEMORY,
];

#[test]
fn finished_is_absorbing() {
    for report in walks() {
        assert_eq!(report.broken_ended(), NONE, "ended runs stay ended");
        assert_ne!(report.found.absorbed, 0, "some run ended");
    }
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
    assert_eq!(report().broken(group::NO_GOAL), NONE, "no goal");
}

#[test]
fn memory_and_notes_only_grow() {
    assert_eq!(report().broken(group::MEMORY), NONE, "memory");
}

#[test]
fn stages_and_explore_mode_keep_the_shared_properties() {
    for report in [staged(), explored()] {
        for group in SHARED {
            assert_eq!(report.broken(group), NONE, "group {group}");
        }
    }
}

#[test]
fn exploration_covers_every_row() {
    let report = report();
    assert_eq!(report.missed(), Vec::<&str>::new(), "every row is reached");
    for report in walks() {
        let nodes = report.found.parents.len();
        assert!((1000..=MAX).contains(&nodes), "{nodes} states");
    }
    // The facts and times that lead to the last state found reach it again.
    let path = report.found.parents.last().copied().flatten().unwrap();
    let (_, trace) = traced((&("the last state", path), report));
    let at = trace.last().map(|(_, at)| *at).unwrap();
    let top = report.candidates.pages.first().unwrap();
    let first = started(&report.rules, top);
    let replayed = trace
        .into_iter()
        .fold(first, |s, (facts, at)| step(s.state().clone(), facts, at));
    assert!(knows(report, &replayed, at), "the replayed state was found");
}

#[test]
fn priority_holds_in_every_reachable_state() {
    for report in walks() {
        assert_eq!(report.broken(group::PRIORITY), NONE, "priority");
        let starts: Vec<bool> = report
            .starts
            .iter()
            .map(|(_, first)| priority::start_holds(first))
            .collect();
        assert!(starts.iter().all(|holds| *holds), "the starts: {starts:?}");
    }
    let staged = staged();
    assert_eq!(staged.missed(), Vec::<&str>::new(), "every stage row");
    let moved_on: Vec<usize> = staged
        .starts
        .iter()
        .map(|(_, first)| first.state().stage())
        .collect();
    assert_eq!(moved_on, [0, 1], "the settings page meets the first stage");
}

#[test]
fn explore_goal_needs_a_question() {
    let explored = explored();
    assert_eq!(explored.broken(group::EXPLORE), NONE, "explore mode");
    let reasons: Vec<Option<ExitReason>> = explored
        .starts
        .iter()
        .map(|(_, first)| first.reason())
        .collect();
    assert_eq!(reasons, [None, None, None], "no start ends the run");
    assert_eq!(explored.missed(), Vec::<&str>::new(), "every explore row");
}
