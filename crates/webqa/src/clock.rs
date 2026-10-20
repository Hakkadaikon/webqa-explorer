//! The real clock: the one place that reads the time.

#[cfg(test)]
mod tests;

use std::future;
use std::time::Duration;

use jiff::Timestamp;
use jiff::tz::TimeZone;
use tokio::time::{Instant, sleep_until};
use webqa_core::ports::{Clock, Pending};

/// The clock of a run. Clones share the start. The run loop starts its own
/// once the browser is up, and the judge's bridge keeps the one it began with.
#[derive(Debug, Clone, Copy)]
pub struct RealClock {
    start: Instant,
}

impl RealClock {
    /// Starts now.
    #[must_use]
    #[expect(
        clippy::disallowed_methods,
        reason = "the one place that reads the time (docs/arch/runtime.md)"
    )]
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Clock for RealClock {
    fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// A time too far to represent is never reached, so the wait never ends.
    #[expect(
        clippy::disallowed_methods,
        reason = "the one place that reads the time (docs/arch/runtime.md)"
    )]
    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        let never: Pending<'_, ()> = Box::pin(future::pending());
        self.start
            .checked_add(at)
            .map_or(never, |deadline| Box::pin(sleep_until(deadline)))
    }
}

/// The wall-clock start of a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wall {
    /// RFC 3339 in UTC to the second, for `run_started`: `2026-10-18T02:00:00Z`.
    pub started_at: String,
    /// The run's directory name, to the millisecond: `20261018T020000.123Z`.
    pub run_id: String,
}

impl Wall {
    /// Reads the wall clock once.
    #[must_use]
    #[expect(
        clippy::disallowed_methods,
        reason = "the one place that reads the time (docs/arch/runtime.md)"
    )]
    pub fn now() -> Self {
        Self::at(Timestamp::now())
    }

    /// The two strings of `at`.
    #[must_use]
    pub fn at(at: Timestamp) -> Self {
        let utc = at.to_zoned(TimeZone::UTC);
        Self {
            started_at: utc.strftime("%Y-%m-%dT%H:%M:%SZ").to_string(),
            run_id: utc.strftime("%Y%m%dT%H%M%S%.3fZ").to_string(),
        }
    }
}
