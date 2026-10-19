//! What tells the states of the search apart.

use std::time::Duration;

use webqa_core::effect::{Effect, PageEffect};
use webqa_core::machine::{State, Step};
use webqa_core::screen::ElementId;

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
/// start screen, the element passed over, the memory, the time and the last
/// effect with the element it points at. Not the notes, which only grow.
///
/// The `Debug` of an element hides its name, so the elements go into the
/// key by their role, name and position.
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
        "{:?} {:?} {} {} {} {} {:?} {} {:?} {:?} {} {} {marks:?} {at:?} {effect:?} {:?}",
        state.phase(),
        state.counters(),
        state.budget().used(),
        window.index(),
        window.used(at),
        state.outside(),
        candidates.page_of(state.last()),
        state.stage(),
        state.origin(),
        state.unmoved().map(named),
        memory.screen_count(),
        memory.edge_count(),
        effect.and_then(target).map(named),
    )
}

/// The element the effect looks up or operates on, if one.
const fn target(effect: &Effect) -> Option<&ElementId> {
    match effect {
        Effect::Page(PageEffect::Look(look)) => Some(&look.element),
        Effect::Page(PageEffect::Touch(touch)) => touch.target.as_ref(),
        Effect::Outside(_) => None,
    }
}

/// The element by its role, name and position.
fn named(element: &ElementId) -> (&str, &str, usize) {
    (&element.role, &element.name, element.nth)
}
