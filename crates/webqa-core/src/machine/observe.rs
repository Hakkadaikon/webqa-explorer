//! Taking an observation into the state, and the rows that take one.

use std::collections::BTreeSet;
use std::time::Duration;

use super::Seen;
use super::actions::{Next, idle};
use super::counters::{Counters, Progress};
use super::material::forbidden_indices;
use super::note::left;
use super::state::State;
use crate::explore::ScreenKey;
use crate::guard::domain::is_allowed;
use crate::screen::{ElementId, changed, element_ids, in_viewport};

/// What the observation told, for counting after an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Taken {
    /// The screen changed, or the page left the allowed origins.
    pub(super) changed: bool,
    /// An untouched, unforbidden element showed in the viewport for the first time.
    pub(super) revealed: bool,
    /// The screen of the previous observation in the memory.
    pub(super) from: Option<ScreenKey>,
}

/// Takes `seen` in as the latest observation.
///
/// Whether the page is outside is judged on the URL before redaction.
pub(super) fn take(mut state: State, seen: Seen) -> (State, Taken) {
    let from = state.at;
    let outside = !is_allowed(&seen.material.paywall.url, &state.rules.allowed);
    let forbidden = forbidden_indices(&state.rules, &seen);
    state.at = (!outside).then(|| state.memory.visit(&seen.screen, &forbidden));
    let revealed = remember(&mut state, &seen, &forbidden);
    let changed = outside || changed(&state.last.screen, &seen.screen, seen.viewport);
    state.last = Box::new(seen);
    state.outside = outside;
    let taken = Taken {
        changed,
        revealed,
        from,
    };
    (state, taken)
}

/// Remembers the elements inside the viewport as shown. True when one of
/// them was not shown before, and is neither operated nor forbidden.
fn remember(state: &mut State, seen: &Seen, forbidden: &BTreeSet<usize>) -> bool {
    let unseen = state
        .at
        .map(|at| unseen(state, seen, at))
        .unwrap_or_default();
    let revealed = unseen
        .iter()
        .any(|(i, at, id)| !forbidden.contains(i) && !state.memory.is_operated(*at, id));
    state
        .shown
        .extend(unseen.into_iter().map(|(_, at, id)| (at, id)));
    revealed
}

/// The elements inside the viewport not shown on the screen `at` before,
/// with their positions in `screen.elements`.
fn unseen(state: &State, seen: &Seen, at: ScreenKey) -> Vec<(usize, ScreenKey, ElementId)> {
    let screen = seen.screen.as_inner();
    element_ids(&seen.screen)
        .into_iter()
        .zip(&screen.elements)
        .enumerate()
        .filter(|(_, (id, e))| in_viewport(e, seen.viewport) && !shown(state, at, id))
        .map(|(i, (id, _))| (i, at, id))
        .collect()
}

/// Whether `id` has been inside the viewport on the screen `at`.
fn shown(state: &State, at: ScreenKey, id: &ElementId) -> bool {
    state.shown.iter().any(|(k, e)| *k == at && e == id)
}

/// Takes `seen` in after an effect. Landing outside the allowed origins
/// records the issue, once for each arrival.
fn landed(state: State, seen: Seen) -> (State, Taken) {
    let (mut state, taken) = take(state, seen);
    if state.outside {
        let issue = left(&state.last);
        state = state.noted(issue);
    }
    (state, taken)
}

/// Takes `seen` in without an operation to count. Landing outside counts
/// as a step without progress, so that a page that keeps sending the run
/// outside ends it stuck instead of returning forever.
pub(super) fn settled(state: State, seen: Seen) -> State {
    let (mut state, _) = landed(state, seen);
    if state.outside {
        state.counters = stalled(state.counters);
    }
    state
}

/// The counters with one more step without progress, the freeze count kept.
fn stalled(counters: Counters) -> Counters {
    counted(counters.freeze(), counters.stuck().saturating_add(1))
}

/// Counters with `freeze` and `stuck`. `Counters` only moves by touches,
/// so the counts are built up from zero by touches.
pub(super) fn counted(freeze: u32, stuck: u32) -> Counters {
    let step = |changed, explored| Progress {
        changed,
        explored,
        revealed: false,
    };
    let changing = (0..stuck.saturating_sub(freeze)).map(|_| step(true, false));
    let exploring = (0..freeze.saturating_sub(stuck)).map(|_| step(false, true));
    let still = (0..freeze.min(stuck)).map(|_| step(false, false));
    changing
        .chain(exploring)
        .chain(still)
        .fold(Counters::default(), Counters::after_touch)
}

/// Takes the page in after returning or reloading.
pub(super) fn arrived(state: State, seen: Seen, _: Duration) -> Next {
    idle(settled(state, seen))
}

/// Takes the page in after an operation and counts what came of it.
pub(super) fn executed(state: State, seen: Seen, _: Duration) -> Next {
    let (mut state, taken) = landed(state, seen);
    let explored = explored(&mut state, taken.from);
    state.counters = state.counters.after_touch(Progress {
        changed: taken.changed,
        explored,
        revealed: taken.revealed,
    });
    idle(state)
}

/// Records the operation on the element in the memory. True when the
/// element was untouched. The page landed on outside counts as staying.
fn explored(state: &mut State, from: Option<ScreenKey>) -> bool {
    let proposal = state.proposal.as_deref();
    let kind = proposal.and_then(|p| p.row).and_then(|row| row.element);
    let element = proposal.and_then(|p| p.element.clone());
    let to = state.at;
    from.zip(kind)
        .zip(element)
        .is_some_and(|((from, kind), element)| {
            state
                .memory
                .record(from, &element, kind, to.unwrap_or(from))
        })
}
