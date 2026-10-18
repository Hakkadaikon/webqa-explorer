//! The freeze and stuck counters and the decision budget.

#[cfg(test)]
mod flow_tests;
#[cfg(test)]
mod tests;

use std::num::NonZeroU32;

/// The number of steps without a screen change (freeze) and without
/// progress in exploration (stuck).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counters {
    freeze: u32,
    stuck: u32,
}

/// What the observation after a browser operation (leaving included) tells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// The screen changed (also true on leaving the allowed domains).
    pub changed: bool,
    /// The set of explored elements grew.
    pub explored: bool,
    /// An untouched element newly appeared in the viewport.
    pub revealed: bool,
}

impl Progress {
    /// Either `explored` or `revealed`.
    #[must_use]
    pub const fn progressed(self) -> bool {
        self.explored || self.revealed
    }
}

impl Counters {
    /// The steps in a row without a screen change.
    #[must_use]
    pub const fn freeze(self) -> u32 {
        self.freeze
    }

    /// The steps in a row without progress in exploration.
    #[must_use]
    pub const fn stuck(self) -> u32 {
        self.stuck
    }

    /// Updates both counts from the observation after an operation.
    #[must_use]
    pub const fn after_touch(self, progress: Progress) -> Self {
        Self {
            freeze: next_count(self.freeze, progress.changed),
            stuck: next_count(self.stuck, progress.progressed()),
        }
    }

    /// After reopening the page: the freeze count restarts, the stuck count stays.
    #[must_use]
    pub const fn after_reload(self) -> Self {
        Self {
            freeze: 0,
            stuck: self.stuck,
        }
    }

    /// Whether the freeze count has reached `threshold`.
    #[must_use]
    pub const fn frozen(self, threshold: NonZeroU32) -> bool {
        self.freeze >= threshold.get()
    }

    /// Whether the stuck count has reached `threshold`.
    #[must_use]
    pub const fn is_stuck(self, threshold: NonZeroU32) -> bool {
        self.stuck >= threshold.get()
    }
}

/// The decision budget. Each query uses one, a malformed answer included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    used: u32,
    cap: NonZeroU32,
}

impl Budget {
    /// Nothing used. `cap` is `limits.max_decisions`.
    #[must_use]
    pub const fn new(cap: NonZeroU32) -> Self {
        Self { used: 0, cap }
    }

    /// Uses one. The cap is not checked here; the state machine guards it.
    #[must_use]
    pub const fn spend(self) -> Self {
        Self {
            used: self.used.saturating_add(1),
            cap: self.cap,
        }
    }

    /// The decisions used.
    #[must_use]
    pub const fn used(self) -> u32 {
        self.used
    }

    /// The cap.
    #[must_use]
    pub const fn cap(self) -> NonZeroU32 {
        self.cap
    }

    /// Whether the budget is used up.
    #[must_use]
    pub const fn exhausted(self) -> bool {
        self.used >= self.cap.get()
    }

    /// The decisions left, zero when more than the cap were used.
    #[must_use]
    pub const fn remaining(self) -> u32 {
        self.cap.get().saturating_sub(self.used)
    }
}

/// The next count: zero when `reset`, one more otherwise.
const fn next_count(count: u32, reset: bool) -> u32 {
    if reset { 0 } else { count.saturating_add(1) }
}
