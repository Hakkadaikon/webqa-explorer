//! The loop: starts a trial when the window and the clock allow it, and
//! waits otherwise.

use std::time::Duration;

use webqa_core::event::{Event, Mode};
use webqa_core::interview::schedule::{Next, RunState, Schedule, next_trial};
use webqa_core::ports::{Clock, Judge, Store};
use webqa_core::redact::Redactor;
use webqa_core::window::RequestWindow;

use super::record::Trial;
use super::write::{Closed, Closing, records};
use super::{Ending, Launch, Plan, Stopped};
use crate::interrupt::{Interrupt, guard};
use crate::interview::reset::ResetError;
use crate::run::{Kept, RunError};

/// The ports of the loop, besides the interrupt.
pub(super) struct Parts<'a> {
    pub(super) launch: &'a mut dyn Launch,
    pub(super) judge: &'a mut dyn Judge,
    pub(super) clock: &'a mut dyn Clock,
    pub(super) store: &'a mut dyn Store,
    pub(super) redactor: &'a Redactor,
}

/// Why the loop stops with nothing more to start.
pub(super) enum Halt {
    /// The reset command failed.
    Reset(ResetError),
    /// A browser or a record failed.
    Run(RunError),
}

impl From<RunError> for Halt {
    fn from(error: RunError) -> Self {
        Self::Run(error)
    }
}

/// Where the run is.
pub(super) struct Course<'a> {
    pub(super) parts: Parts<'a>,
    pub(super) interrupt: &'a mut dyn Interrupt,
    pub(super) plan: &'a Plan,
    /// The run's window: the first pages and the core's accesses.
    pub(super) window: RequestWindow,
    /// Whether the user interrupted the run.
    pub(super) interrupted: bool,
    /// The trials started, in order.
    pub(super) trials: Vec<Trial>,
    /// What the run keeps from one browser to the next.
    pub(super) kept: Kept,
}

/// Runs the trials, then writes the records.
pub(super) async fn conduct<'a>(
    parts: Parts<'a>,
    interrupt: &'a mut dyn Interrupt,
    plan: &'a Plan,
) -> Ending {
    let mut course = Course {
        window: RequestWindow::new(plan.interview.requests_per_hour),
        parts,
        interrupt,
        plan,
        interrupted: false,
        trials: Vec::new(),
        kept: Kept::default(),
    };
    match course.go().await {
        Ok(stopped) => course.finish(stopped, None).await,
        Err(Halt::Reset(error)) => course.finish(Stopped::Completed, Some(error)).await,
        Err(Halt::Run(error)) => Ending::Failed(error),
    }
}

impl Course<'_> {
    /// Starts the log, then runs the trials.
    async fn go(&mut self) -> Result<Stopped, Halt> {
        let started = Event::RunStarted {
            schema: 1,
            mode: Mode::Scenario,
            started_at: None,
        };
        self.log(started).await?;
        self.start_trials().await
    }

    /// Starts and waits until no more trials start.
    async fn start_trials(&mut self) -> Result<Stopped, Halt> {
        let mut stopped = self.step().await?;
        while stopped.is_none() {
            stopped = self.step().await?;
        }
        // The loop only ends with a reason; the default keeps the type total.
        Ok(stopped.unwrap_or(Stopped::Completed))
    }

    /// Starts a trial, waits, or says why no more trials start.
    async fn step(&mut self) -> Result<Option<Stopped>, Halt> {
        match self.next() {
            Next::Stop(stop) => Ok(Some(stop.into())),
            Next::Wait(at) => {
                self.wait(at).await;
                Ok(None)
            }
            Next::Start(cap) => self.trial(cap).await.map(|()| None),
        }
    }

    /// Writes `event` as the next line of the log.
    pub(super) async fn log(&mut self, event: Event) -> Result<(), RunError> {
        let elapsed = self.parts.clock.elapsed();
        let store = &mut *self.parts.store;
        let journal = &mut self.kept.journal;
        journal
            .write(None, event, elapsed, store, self.parts.redactor)
            .await
    }

    /// What to do next, from the trials started and the window.
    fn next(&self) -> Next {
        let interview = &self.plan.interview;
        let schedule = Schedule {
            trials: interview.trials,
            max_duration: self.plan.max_duration,
        };
        let started = u32::try_from(self.trials.len()).unwrap_or(u32::MAX);
        let state = RunState {
            started,
            interrupted: self.interrupted,
            window: self.window,
        };
        next_trial(schedule, state, self.parts.clock.elapsed())
    }

    /// Sleeps until `at`, or until the user interrupts.
    async fn wait(&mut self, at: Duration) {
        let interrupt = &mut *self.interrupt;
        let clock = &mut *self.parts.clock;
        let interrupted = tokio::select! {
            biased;
            () = interrupt.wait() => true,
            () = clock.sleep_until(at) => false,
        };
        self.interrupted |= interrupted;
    }

    /// Writes the records, unless the user interrupts again. `failed` is the
    /// reset command's failure that stopped the loop, if one did.
    async fn finish(&mut self, stopped: Stopped, failed: Option<ResetError>) -> Ending {
        let allowance = u8::from(stopped != Stopped::Interrupted);
        let closing = Closing {
            judge: &mut *self.parts.judge,
            clock: &mut *self.parts.clock,
            store: &mut *self.parts.store,
            redactor: self.parts.redactor,
            kept: &mut self.kept,
            trials: &self.trials,
            plan: self.plan,
        };
        let closed = guard(records(closing, stopped), &mut *self.interrupt, allowance);
        match closed.await {
            None => Ending::Abandoned,
            Some(Err(error)) => Ending::Failed(error),
            Some(Ok(Closed::Mismatch(mismatch))) => Ending::Mismatch(mismatch),
            Some(Ok(Closed::Wrote { confirmed })) => {
                failed.map_or(Ending::Wrote { stopped, confirmed }, Ending::ResetFailed)
            }
        }
    }
}
