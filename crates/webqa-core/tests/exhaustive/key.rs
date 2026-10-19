//! What tells the states of the search apart.

use std::time::Duration;

use webqa_core::effect::Effect;
use webqa_core::machine::{State, Step};

use super::candidates::Candidates;
use super::report::Report;
use super::view::marks;

/// Whether replaying a trace that ends at `at` with `step` reaches a state the search found.
pub fn knows(report: &Report, step: &Step, at: Duration) -> bool {
    let state = step.state();
    let candidates = &report.candidates;
    let marks = marks(state, &candidates.marked);
    report
        .found
        .keys
        .contains(&key(candidates, state, &marks, step.effect(), at))
}

/// What tells states apart: the counts, where the run is, the stage, the
/// start screen, the memory, the time and the last effect. Not the notes,
/// which only grow.
pub fn key(
    candidates: &Candidates,
    state: &State,
    marks: &[Vec<bool>],
    effect: Option<&Effect>,
    at: Duration,
) -> String {
    let window = state.window();
    let memory = state.memory();
    format!(
        "{:?} {:?} {} {} {} {} {:?} {} {:?} {} {} {marks:?} {at:?} {effect:?}",
        state.phase(),
        state.counters(),
        state.budget().used(),
        window.index(),
        window.used(at),
        state.outside(),
        candidates.page_of(state.last()),
        state.stage(),
        state.origin(),
        memory.screen_count(),
        memory.edge_count(),
    )
}
