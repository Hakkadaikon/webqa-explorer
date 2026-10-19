//! The interrupts and the clock of the tests of the loop.

use std::future;
use std::time::Duration;

use webqa_core::ports::{Clock, Pending};

use crate::interrupt::Interrupt;

/// The user who never interrupts.
pub(super) struct Never;

impl Interrupt for Never {
    fn wait(&mut self) -> Pending<'_, ()> {
        Box::pin(future::pending())
    }
}

/// Interrupts on the calls of `wait` numbered in `fire_on`, from 1.
pub(super) struct Fires {
    pub(super) calls: usize,
    pub(super) fire_on: Vec<usize>,
}

impl Interrupt for Fires {
    fn wait(&mut self) -> Pending<'_, ()> {
        self.calls += 1;
        if self.fire_on.contains(&self.calls) {
            Box::pin(future::ready(()))
        } else {
            Box::pin(future::pending())
        }
    }
}

/// A clock that stands at `now` and never wakes from a sleep.
pub(super) struct Stalled {
    pub(super) now: Duration,
    pub(super) sleeps: Vec<Duration>,
}

impl Clock for Stalled {
    fn elapsed(&self) -> Duration {
        self.now
    }

    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        self.sleeps.push(at);
        Box::pin(future::pending())
    }
}
