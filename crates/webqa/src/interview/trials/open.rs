//! Before a trial: the reset command, and the browser.

use std::future;
use std::time::Duration;

use webqa_core::ports::{Browser, BrowserError, Clock};

use super::course::{Course, Halt};
use crate::interrupt::guard;
use crate::interview::reset::{self, ResetError};
use crate::run::RunError;

/// What got in the way of opening the browser first, or the browser.
enum Raced {
    /// The user interrupted.
    Interrupted,
    /// The time limit passed.
    TimeUp,
    /// The launch returned.
    Launched(Result<Box<dyn Browser>, BrowserError>),
}

impl Course<'_> {
    /// Resets the service and opens the browser of the next trial. `None`
    /// when the user interrupted or the time limit passed first: the trial
    /// does not start, and the loop decides what comes next.
    pub(super) async fn opening(&mut self) -> Result<Option<Box<dyn Browser>>, Halt> {
        if !self.reset().await? {
            return Ok(None);
        }
        Ok(self.open().await?)
    }

    /// Runs the reset command, if the settings have one. Whether to go on:
    /// not after an interruption, nor when the time limit passed while the
    /// command ran, whatever came of it.
    async fn reset(&mut self) -> Result<bool, Halt> {
        let Some(command) = &self.plan.interview.reset else {
            return Ok(true);
        };
        let withheld: Vec<&str> = self.plan.withheld.iter().map(String::as_str).collect();
        let clock = &mut *self.parts.clock;
        let running = reset::run(
            command,
            &self.plan.env,
            &withheld,
            self.parts.redactor,
            clock,
        );
        let Some(ran) = guard(running, &mut *self.interrupt, 0).await else {
            self.interrupted = true;
            return Ok(false);
        };
        self.after_reset(ran.result)
    }

    /// Whether to go on after the reset command ended with `result`.
    fn after_reset(&self, result: Result<(), ResetError>) -> Result<bool, Halt> {
        if self.time_is_up() {
            Ok(false)
        } else {
            result.map(|()| true).map_err(Halt::Reset)
        }
    }

    /// Whether the time limit has passed.
    fn time_is_up(&self) -> bool {
        let limit = self.plan.max_duration;
        limit.is_some_and(|limit| self.parts.clock.elapsed() >= limit)
    }

    /// Opens the browser, unless the user interrupts or the time limit
    /// passes first.
    async fn open(&mut self) -> Result<Option<Box<dyn Browser>>, RunError> {
        let launching = self.parts.launch.launch();
        let interrupt = &mut *self.interrupt;
        let late = until(&mut *self.parts.clock, self.plan.max_duration);
        let raced = tokio::select! {
            biased;
            () = interrupt.wait() => Raced::Interrupted,
            launched = launching => Raced::Launched(launched),
            () = late => Raced::TimeUp,
        };
        match raced {
            Raced::Launched(launched) => Ok(Some(launched?)),
            Raced::Interrupted => {
                self.interrupted = true;
                Ok(None)
            }
            Raced::TimeUp => Ok(None),
        }
    }
}

/// Waits until `limit` on `clock`, and never when there is no limit.
async fn until(clock: &mut dyn Clock, limit: Option<Duration>) {
    match limit {
        Some(at) => clock.sleep_until(at).await,
        None => future::pending().await,
    }
}
