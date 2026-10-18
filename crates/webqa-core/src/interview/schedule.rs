//! Whether the next trial starts, and the cap on the core's window.

#[cfg(test)]
mod tests;

use std::num::NonZeroU32;
use std::time::Duration;

use crate::window::{RequestWindow, wake_at};

/// What does not change during a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Schedule {
    /// How many trials to run (`interview.trials`).
    pub trials: NonZeroU32,
    /// The time limit from the start of the run (`limits.max_duration_secs`).
    /// None when there is no limit.
    pub max_duration: Option<Duration>,
}

/// Where the run is when deciding on the next trial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunState {
    /// The trials started so far: finished ones and an aborted one together.
    pub started: u32,
    /// Whether the user interrupted the run.
    pub interrupted: bool,
    /// The run window, with the first pages and the core's accesses counted.
    pub window: RequestWindow,
}

/// Why no more trials start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    /// The user interrupted the run.
    Interrupted,
    /// The time limit has passed.
    TimeUp,
    /// All the trials have started.
    Completed,
}

/// What the loop does next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    /// Start a trial now, with this cap on the core's window.
    Start(NonZeroU32),
    /// Start nothing yet. Wake at this time and decide again.
    Wait(Duration),
    /// Start no more trials.
    Stop(Stop),
}

/// The run window after the first page of a trial is shown at `now`.
/// The first page counts as one access. The cap is not checked.
#[must_use]
pub const fn count_first_page(run: RequestWindow, now: Duration) -> RequestWindow {
    run.spend(now)
}

/// Whether to start the next trial at `now`, and with what cap on the core's window.
/// Checks the interruption, the time limit, the number of trials and the
/// run window in this order.
#[must_use]
pub fn next_trial(schedule: Schedule, state: RunState, now: Duration) -> Next {
    stop_reason(schedule, state, now).map_or_else(
        || {
            core_cap(state.window, now).map_or_else(
                || Next::Wait(wake_at(state.window.next_start(now), schedule.max_duration)),
                Next::Start,
            )
        },
        Next::Stop,
    )
}

/// The core's window for a trial started at `now` with `cap`.
/// It is counted from the start of the run, so it turns over at the same
/// times as the run window. Nothing is used yet.
#[must_use]
pub const fn core_window(cap: NonZeroU32, now: Duration) -> RequestWindow {
    RequestWindow::new(cap).at(now)
}

/// Why no more trials start, checked in order, or None to go on.
fn stop_reason(schedule: Schedule, state: RunState, now: Duration) -> Option<Stop> {
    if state.interrupted {
        Some(Stop::Interrupted)
    } else if time_up(schedule.max_duration, now) {
        Some(Stop::TimeUp)
    } else if all_started(schedule.trials, state.started) {
        Some(Stop::Completed)
    } else {
        None
    }
}

/// Whether the time limit has passed. Reaching it exactly counts as passed.
fn time_up(max_duration: Option<Duration>, now: Duration) -> bool {
    max_duration.is_some_and(|limit| now >= limit)
}

/// Whether every trial has started.
const fn all_started(trials: NonZeroU32, started: u32) -> bool {
    started >= trials.get()
}

/// The cap for the core: what remains in the window at `now` less one for
/// the first page, when that is at least half the run's cap.
fn core_cap(window: RequestWindow, now: Duration) -> Option<NonZeroU32> {
    let cap = window.cap().get();
    let left = cap.saturating_sub(window.used(now)).saturating_sub(1);
    let at_half = u64::from(left).saturating_mul(2) >= u64::from(cap);
    NonZeroU32::new(left).filter(|_| at_half)
}
