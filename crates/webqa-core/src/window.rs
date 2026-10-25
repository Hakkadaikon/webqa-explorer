//! The hourly window that limits accesses to the target server.

#[cfg(test)]
mod flow_tests;
#[cfg(test)]
mod tests;

use std::num::NonZeroU32;
use std::time::Duration;

/// The length of a window. Windows are counted hourly from the start of the run.
pub const WINDOW: Duration = Duration::from_secs(3600);

/// The current window and the accesses to the target server made in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestWindow {
    index: u64,
    used: u32,
    cap: NonZeroU32,
}

impl RequestWindow {
    /// Window 0 with nothing used. `cap` is `limits.requests_per_hour`.
    #[must_use]
    pub const fn new(cap: NonZeroU32) -> Self {
        Self {
            index: 0,
            used: 0,
            cap,
        }
    }

    /// The number of the window that the elapsed time `now` falls in.
    #[must_use]
    pub const fn index_at(now: Duration) -> u64 {
        now.as_secs() / WINDOW.as_secs()
    }

    /// The value for the window at `now`. A later window starts empty;
    /// an earlier time (a clock going back) leaves the value as it is.
    #[must_use]
    pub const fn at(self, now: Duration) -> Self {
        let index = Self::index_at(now);
        if index > self.index {
            Self {
                index,
                used: 0,
                cap: self.cap,
            }
        } else {
            self
        }
    }

    /// The accesses used in the window at `now`.
    #[must_use]
    pub const fn used(self, now: Duration) -> u32 {
        self.at(now).used
    }

    /// Whether the window at `now` has reached the cap.
    #[must_use]
    pub const fn full(self, now: Duration) -> bool {
        self.used(now) >= self.cap.get()
    }

    /// Moves to the window at `now` and uses one access. The cap is not checked.
    #[must_use]
    pub const fn spend(self, now: Duration) -> Self {
        let current = self.at(now);
        Self {
            used: current.used.saturating_add(1),
            ..current
        }
    }

    /// The start of the window after the one at `now`.
    #[must_use]
    pub const fn next_start(self, now: Duration) -> Duration {
        Duration::from_secs(
            Self::index_at(now)
                .saturating_add(1)
                .saturating_mul(WINDOW.as_secs()),
        )
    }

    /// The number of the recorded window, not moved to any time.
    #[must_use]
    pub const fn index(self) -> u64 {
        self.index
    }

    /// The cap.
    #[must_use]
    pub const fn cap(self) -> NonZeroU32 {
        self.cap
    }
}
