//! What the core keeps between effects, and how a run ends.

use serde::{Deserialize, Serialize};

use super::Seen;
use super::counters::{Budget, Counters};
use super::note::Note;
use super::observe::take;
use super::proposal::Proposal;
use super::rules::Rules;
use crate::explore::{Memory, ScreenKey};
use crate::screen::ElementId;
use crate::window::RequestWindow;

/// Where the run is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Waiting. Only seen while the table is being run, never in a returned state.
    Idle,
    /// Waiting for the judge's answers.
    Awaiting,
    /// Waiting for the element to be looked up again.
    Confirming,
    /// Waiting for the page after an operation or leaving a paywall.
    Acting,
    /// Waiting for the page after returning to an allowed origin.
    Returning,
    /// Waiting for the page after a reload.
    Recovering,
    /// Waiting for the time window to open.
    Sleeping,
    /// The run is over.
    Ended(ExitReason),
}

/// Why the run ended. Serialized as `{"reason": "<name>"}`, and stuck adds `"kind"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", content = "kind", rename_all = "snake_case")]
pub enum ExitReason {
    /// The goal is achieved.
    GoalAchieved,
    /// The judge said it is done. Not verified.
    ReportedDone,
    /// Stuck, in one of two ways.
    Stuck(StuckKind),
    /// The decision budget is used up.
    BudgetExceeded,
    /// The time limit is reached.
    TimeExceeded,
    /// The user interrupted the run.
    UserAborted,
}

/// How the run got stuck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StuckKind {
    /// The judge answered `stuck`.
    Declared,
    /// The stuck count reached its threshold.
    NoProgress,
}

impl ExitReason {
    /// The kind of being stuck, if stuck.
    #[must_use]
    pub const fn stuck_kind(self) -> Option<StuckKind> {
        if let Self::Stuck(kind) = self {
            Some(kind)
        } else {
            None
        }
    }

    /// Whether stuck, of either kind.
    #[must_use]
    pub const fn is_stuck(self) -> bool {
        self.stuck_kind().is_some()
    }
}

/// Who asked for the browser operation a state waits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    /// The judge's proposal, passed by the guardrails.
    Judge,
    /// Leaving a paywall.
    Paywall,
    /// Returning to an allowed origin.
    Outside,
    /// Reloading a frozen page.
    Frozen,
}

/// What the core keeps between effects.
///
/// `Debug` shows no page strings: the screen, the material, the memory and
/// the notes hide them.
#[derive(Debug, Clone)]
pub struct State {
    pub(super) rules: Rules,
    pub(super) phase: Phase,
    pub(super) last: Box<Seen>,
    /// The screen of `last` in the memory. `None` outside the allowed origins.
    pub(super) at: Option<ScreenKey>,
    pub(super) memory: Memory,
    /// The elements that have been inside the viewport, by screen.
    pub(super) shown: Vec<(ScreenKey, ElementId)>,
    pub(super) counters: Counters,
    pub(super) budget: Budget,
    pub(super) window: RequestWindow,
    pub(super) outside: bool,
    /// Held from the judge's answer until the operation is observed.
    pub(super) proposal: Option<Box<Proposal>>,
    pub(super) notes: Vec<Note>,
    /// The current stage of the scenario, from 0. Stays 0 in explore mode.
    pub(super) stage: usize,
    /// The start screen in the memory: the first one taken in inside the
    /// allowed origins.
    pub(super) origin: Option<ScreenKey>,
    /// The element of the last operation, when the screen did not change
    /// after it. Cleared by every observation taken in.
    pub(super) unmoved: Option<ElementId>,
}

impl State {
    /// Waiting, with the first observation `seen` taken in.
    pub(super) fn observed(rules: Rules, seen: Seen, memory: Memory) -> Self {
        let budget = Budget::new(rules.max_decisions);
        let window = RequestWindow::new(rules.requests_per_hour);
        let blank = Self {
            rules,
            phase: Phase::Idle,
            last: Box::new(seen.clone()),
            at: None,
            memory,
            shown: Vec::new(),
            counters: Counters::default(),
            budget,
            window,
            outside: false,
            proposal: None,
            notes: Vec::new(),
            stage: 0,
            origin: None,
            unmoved: None,
        };
        take(blank, seen).0
    }

    /// Where the run is.
    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    /// The latest observation taken in.
    #[must_use]
    pub fn last(&self) -> &Seen {
        &self.last
    }

    /// The exploration memory.
    #[must_use]
    pub const fn memory(&self) -> &Memory {
        &self.memory
    }

    /// The freeze and stuck counts.
    #[must_use]
    pub const fn counters(&self) -> Counters {
        self.counters
    }

    /// The decision budget.
    #[must_use]
    pub const fn budget(&self) -> Budget {
        self.budget
    }

    /// The time window.
    #[must_use]
    pub const fn window(&self) -> RequestWindow {
        self.window
    }

    /// Whether the latest page is outside the allowed origins.
    #[must_use]
    pub const fn outside(&self) -> bool {
        self.outside
    }

    /// What the core recorded, oldest first. It only grows.
    #[must_use]
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    /// The rules of the run.
    #[must_use]
    pub const fn rules(&self) -> &Rules {
        &self.rules
    }

    /// The current stage of the scenario, from 0. Always 0 in explore mode.
    #[must_use]
    pub const fn stage(&self) -> usize {
        self.stage
    }

    /// The start screen in the memory, once a page inside the allowed
    /// origins has been taken in.
    #[must_use]
    pub const fn origin(&self) -> Option<ScreenKey> {
        self.origin
    }

    /// The element of the last operation, when the screen did not change
    /// after it.
    #[must_use]
    pub const fn unmoved(&self) -> Option<&ElementId> {
        self.unmoved.as_ref()
    }

    /// Why the run ended, if it has.
    #[must_use]
    pub const fn ended(&self) -> Option<ExitReason> {
        if let Phase::Ended(reason) = self.phase {
            Some(reason)
        } else {
            None
        }
    }

    /// Who asked for the operation this state waits on. `None` unless it
    /// waits for the page after a browser operation.
    #[must_use]
    pub fn cause(&self) -> Option<Cause> {
        match self.phase {
            Phase::Acting => Some(self.acting()),
            Phase::Returning => Some(Cause::Outside),
            Phase::Recovering => Some(Cause::Frozen),
            Phase::Idle
            | Phase::Awaiting
            | Phase::Confirming
            | Phase::Sleeping
            | Phase::Ended(_) => None,
        }
    }

    /// The judge's proposal when the held one has an operation kind, else
    /// leaving a paywall.
    fn acting(&self) -> Cause {
        if self.proposal.as_ref().is_some_and(|p| p.row.is_some()) {
            Cause::Judge
        } else {
            Cause::Paywall
        }
    }

    /// In `phase`.
    pub(super) const fn to(mut self, phase: Phase) -> Self {
        self.phase = phase;
        self
    }

    /// With `note` recorded.
    pub(super) fn noted(mut self, note: Note) -> Self {
        self.notes.push(note);
        self
    }

    /// With the window replaced.
    pub(crate) const fn with_window(mut self, window: RequestWindow) -> Self {
        self.window = window;
        self
    }
}

#[cfg(test)]
impl State {
    /// With the counters replaced.
    pub(crate) const fn with_counters(mut self, counters: Counters) -> Self {
        self.counters = counters;
        self
    }

    /// With the budget replaced.
    pub(crate) const fn with_budget(mut self, budget: Budget) -> Self {
        self.budget = budget;
        self
    }

    /// In the stage `stage`.
    pub(crate) const fn with_stage(mut self, stage: usize) -> Self {
        self.stage = stage;
        self
    }
}
