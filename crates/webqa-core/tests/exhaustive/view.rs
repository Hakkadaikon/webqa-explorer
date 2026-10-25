//! One transition of the search, and what the properties read from it.

use std::collections::BTreeSet;
use std::time::Duration;

use webqa_core::effect::{Effect, Outside, PageEffect, Touch};
use webqa_core::explore::Memory;
use webqa_core::machine::{ExitReason, Note, Seen, State, Step};
use webqa_core::screen::{ElementId, element_ids};

/// The state `before`, the facts passed at `at`, and the step returned.
pub struct Transition<'a> {
    /// The state the facts were passed to.
    pub before: &'a State,
    /// The operated marks of `before` on the candidate pages.
    pub before_marks: &'a [Vec<bool>],
    /// The operation kind of the answer passed, if a reply with answers.
    pub op: Option<&'static str>,
    /// The time the facts were passed at.
    pub at: Duration,
    /// What came back.
    pub step: &'a Step,
    /// The operated marks of the state after, on the candidate pages.
    pub after_marks: &'a [Vec<bool>],
    /// The candidate page the state after last observed.
    pub last: Option<usize>,
    /// The candidate page the run started on.
    pub root: usize,
}

/// A property of a transition, by name.
pub struct Property {
    /// What the property says.
    pub name: &'static str,
    /// Whether the transition keeps it.
    pub holds: fn(&Transition) -> bool,
}

/// `b` whenever `a`.
pub const fn implies(a: bool, b: bool) -> bool {
    !a || b
}

/// Every condition holds.
pub fn all(conditions: &[bool]) -> bool {
    conditions.iter().all(|c| *c)
}

/// Some condition holds.
pub fn any(conditions: &[bool]) -> bool {
    conditions.iter().any(|c| *c)
}

/// Whether the judge is asked.
pub const fn asks(t: &Transition) -> bool {
    matches!(t.step.effect(), Some(Effect::Outside(Outside::Ask(_))))
}

/// Whether the run sleeps.
pub const fn sleeps(t: &Transition) -> bool {
    matches!(t.step.effect(), Some(Effect::Outside(Outside::Sleep(_))))
}

/// When the run wakes, if it sleeps.
pub const fn wake(t: &Transition) -> Option<Duration> {
    if let Some(Effect::Outside(Outside::Sleep(wake))) = t.step.effect() {
        Some(wake.at)
    } else {
        None
    }
}

/// Whether an element is looked up again.
pub const fn looks(t: &Transition) -> bool {
    matches!(t.step.effect(), Some(Effect::Page(PageEffect::Look(_))))
}

/// For each candidate page, whether the memory of `state` has each of its
/// elements operated.
pub fn marks(state: &State, pages: &[Seen]) -> Vec<Vec<bool>> {
    let mut memory = state.memory().clone();
    pages.iter().map(|seen| marked(&mut memory, seen)).collect()
}

/// Visiting a page in a copy of the memory adds no marks to the others.
fn marked(memory: &mut Memory, seen: &Seen) -> Vec<bool> {
    let at = memory.visit(&seen.screen, &BTreeSet::new());
    element_ids(&seen.screen)
        .iter()
        .map(|id| memory.is_operated(at, id))
        .collect()
}

impl Transition<'_> {
    /// The state after: the one to go on with, or the last one.
    pub fn after(&self) -> &State {
        self.step.state()
    }

    /// Why the run ended, if it did.
    pub const fn reason(&self) -> Option<ExitReason> {
        self.step.reason()
    }

    /// The browser operation asked for, if one.
    pub const fn touch(&self) -> Option<&Touch> {
        if let Some(Effect::Page(PageEffect::Touch(touch))) = self.step.effect() {
            Some(touch)
        } else {
            None
        }
    }

    /// The element of the browser operation asked for, if one.
    pub fn target(&self) -> Option<&ElementId> {
        self.touch().and_then(|touch| touch.target.as_ref())
    }

    /// Whether the browser operation `op` is asked for.
    pub fn touches(&self, op: &str) -> bool {
        self.touch().is_some_and(|touch| touch.op == op)
    }

    /// Whether the transition recorded `note`.
    pub fn adds(&self, note: &Note) -> bool {
        self.after()
            .notes()
            .iter()
            .skip(self.before.notes().len())
            .any(|n| n == note)
    }
}
