//! Taking an observation into the state, and the rows that take one.

use std::collections::BTreeSet;

use super::Seen;
use super::actions::{Next, Now, idle, page};
use super::counters::Progress;
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

/// Takes the page in after returning or reloading, without counting.
pub(super) fn arrived(state: State, now: &Now) -> Next {
    let seen = page(&state, now);
    idle(take(state, seen).0)
}

/// Takes the page in after an operation and counts what came of it.
pub(super) fn executed(state: State, now: &Now) -> Next {
    let seen = page(&state, now);
    let (mut state, taken) = take(state, seen);
    let explored = explored(&mut state, taken.from);
    state.counters = state.counters.after_touch(Progress {
        changed: taken.changed,
        explored,
        revealed: taken.revealed,
    });
    if state.outside {
        let issue = left(&state.last);
        state = state.noted(issue);
    }
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
