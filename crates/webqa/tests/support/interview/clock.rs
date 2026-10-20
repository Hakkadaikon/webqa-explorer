//! The clock of an interview run: time moves when the judge thinks, when
//! the run sleeps, and when a reset script writes the time to its file.

use std::fs;
use std::future;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::sync::Notify;
use webqa_core::ports::{Clock, Pending};

use crate::clock::millis;
use crate::fakes::stuck::stuck;

/// How many runs of the reset command the file `out` counts: one line
/// each, as the scripts of the fixtures write them.
pub fn resets(out: Option<&Path>) -> usize {
    let text = out.and_then(|out| fs::read_to_string(out).ok());
    text.map_or(0, |text| text.lines().count())
}

/// The reset command as the clock sees it.
#[derive(Debug, Default)]
pub struct Resets {
    /// How long the run waits for the command. A sleep until that long
    /// from now is the deadline of a run of the command.
    pub timeout: Option<Duration>,
    /// The first run, from 1, that does not finish.
    pub hangs_from: Option<usize>,
    /// The file the command counts its runs in.
    pub out: Option<PathBuf>,
    /// How many deadlines of the command the run has slept until.
    pub deadlines: usize,
}

impl Resets {
    /// Whether a sleep until `at` from `now` is the deadline of a run that
    /// does not finish; `None` when it is no deadline of the command.
    fn deadline(&mut self, now: Duration, at: Duration) -> Option<bool> {
        let timeout = self.timeout?;
        (now + timeout == at).then(|| {
            self.deadlines += 1;
            self.hangs_from.is_some_and(|from| self.deadlines >= from)
        })
    }
}

/// The clock of one run. A sleep ends at once and moves the clock, except
/// the deadline of a reset command that finishes, which never ends so that
/// the command ends first, and any other sleep when `stall_sleep` is set.
#[derive(Debug, Default)]
pub struct TrialClock {
    /// The elapsed time in milliseconds, shared with the judge.
    pub now: Arc<AtomicU64>,
    /// Whether a sleep never ends. It wakes the gate instead.
    pub stall_sleep: bool,
    /// The reset command.
    pub resets: Resets,
    /// The file a reset script writes the elapsed milliseconds to.
    pub file: Option<PathBuf>,
    /// The times the run slept until, in order, each with how many runs of
    /// the reset command had been made when it began.
    pub sleeps: Vec<(Duration, usize)>,
    /// Wakes the interrupt when something waits forever.
    pub gate: Arc<Notify>,
}

impl TrialClock {
    /// How many runs of the reset command were made when a sleep began. A
    /// run may count itself before its deadline is asked for or after, so
    /// for a deadline the deadlines count the runs before it.
    fn made(&self, deadline: Option<bool>) -> usize {
        deadline.map_or_else(
            || resets(self.resets.out.as_deref()),
            |_| self.resets.deadlines.saturating_sub(1),
        )
    }
}

impl Clock for TrialClock {
    fn elapsed(&self) -> Duration {
        let written = self
            .file
            .as_ref()
            .and_then(|file| fs::read_to_string(file).ok());
        if let Some(ms) = written.and_then(|text| text.trim().parse().ok()) {
            self.now.fetch_max(ms, Ordering::SeqCst);
        }
        Duration::from_millis(self.now.load(Ordering::SeqCst))
    }

    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        let now = self.elapsed();
        let deadline = self.resets.deadline(now, at);
        let made = self.made(deadline);
        self.sleeps.push((at, made));
        match deadline {
            Some(false) => stuck(),
            None if self.stall_sleep => {
                self.gate.notify_one();
                Box::pin(future::pending())
            }
            Some(true) | None => {
                self.now.fetch_max(millis(at), Ordering::SeqCst);
                Box::pin(future::ready(()))
            }
        }
    }
}
