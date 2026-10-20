//! The clock of an interview run: time moves when the judge thinks, when
//! the run sleeps, and when a reset script writes the time to its file.

use std::fs;
use std::future;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, PoisonError};
use std::time::Duration;

use tokio::sync::Notify;
use webqa_core::ports::{Clock, Pending};

use crate::clock::millis;
use crate::fakes::stuck::stuck;
use crate::launch::{Opened, Openings};

/// How many runs of the reset command the file `out` counts: one line
/// each, as the scripts of the fixtures write them when they start.
pub fn resets(out: &Path) -> usize {
    fs::read_to_string(out).map_or(0, |text| text.lines().count())
}

/// The file `runs.sh` writes a line to when a run finishes: `out` with
/// `.done` after it.
pub fn done_of(out: &Path) -> PathBuf {
    let mut done = out.as_os_str().to_owned();
    done.push(".done");
    PathBuf::from(done)
}

/// The reset command as the clock sees it.
#[derive(Debug, Default)]
pub struct Resets {
    /// How long the run waits for the command. A sleep until that long
    /// from now, before the browser of the trial the run is for opens, is
    /// the deadline of a run of the command. Another sleep that happens to
    /// end that long from now before that browser opens, such as a window
    /// that clears exactly then, is taken for a deadline too: scenes are
    /// made so that the two do not meet.
    pub timeout: Option<Duration>,
    /// The first run, from 1, that does not finish.
    pub hangs_from: Option<usize>,
    /// The file the command counts its runs in.
    pub out: PathBuf,
    /// How many deadlines of the command the run has slept until.
    pub deadlines: usize,
    /// What happened as the browsers opened. A run of the command comes
    /// before each browser, so there is no deadline between a run and the
    /// browser after it.
    pub openings: Openings,
}

impl Resets {
    /// Whether a sleep until `at` from `now` is the deadline of a run that
    /// does not finish; `None` when it is no deadline of the command.
    fn deadline(&mut self, now: Duration, at: Duration) -> Option<bool> {
        let timeout = self.timeout?;
        let opened = self.launched();
        (now + timeout == at && self.deadlines <= opened).then(|| {
            self.deadlines += 1;
            self.hangs_from.is_some_and(|from| self.deadlines >= from)
        })
    }

    /// How many browsers were asked for.
    fn launched(&self) -> usize {
        let launched = |(_, opened): &&(usize, Opened)| matches!(opened, Opened::Launched { .. });
        let all = self.openings.lock();
        all.unwrap_or_else(PoisonError::into_inner)
            .iter()
            .filter(launched)
            .count()
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
    /// The second item of each of `sleeps`: how many runs of the reset
    /// command had started when the sleep began. For the deadline of a run
    /// that is the runs before it, the deadlines so far less this one, as
    /// the run may write its line to the file before its deadline is asked
    /// for or after. For any other sleep it is the lines of the file.
    fn made(&self, deadline: Option<bool>) -> usize {
        deadline.map_or_else(
            || resets(&self.resets.out),
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
        // Not `elapsed`: the reset command read it before this sleep, and a
        // script may write the clock's file in between.
        let now = Duration::from_millis(self.now.load(Ordering::SeqCst));
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
