//! The loop outside the trials: runs one persona and purpose for the number
//! of trials the settings say, then writes one ledger.
//!
//! Each trial opens a fresh browser and runs the core in scenario mode with
//! the purpose as its one stage. A trial counts for the ledger when it
//! finishes; the time limit or an interruption cuts it and the run.

mod after;
mod course;
mod open;
mod record;
mod shots;
#[cfg(test)]
mod tests;
mod trial;
mod write;

use std::ffi::OsString;
use std::time::Duration;

use webqa_core::config::interview::Interview;
use webqa_core::event::Mode;
use webqa_core::interview::compare::Mismatch;
use webqa_core::interview::ledger::Ledger;
use webqa_core::interview::schedule::Stop;
use webqa_core::machine::Rules;
use webqa_core::ports::{Browser, BrowserError, Clock, Judge, Pending, Store};
use webqa_core::redact::{Redacted, Redactor};

use super::reset::ResetError;
use crate::interrupt::Interrupt;
use crate::run::{Detect, RunError};
use crate::store::report::Price;

/// What the loop needs besides its ports.
pub struct Plan {
    /// The persona, the purpose and the settings of the trials.
    pub interview: Interview,
    /// The core's rules made from the configuration. The loop sets the
    /// mode, the stages and the stuck threshold.
    pub rules: Rules,
    /// The start URL, redacted: the ledger's head says it.
    pub start: Redacted<String>,
    /// The ledger of the run before, whose head was checked against this
    /// run's.
    pub previous: Option<Ledger>,
    /// The time limit from the start of the run.
    pub max_duration: Option<Duration>,
    /// The environment variables of the reset command.
    pub env: Vec<(OsString, OsString)>,
    /// The strings that must not reach the reset command: the judge's API
    /// key.
    pub withheld: Vec<String>,
    /// The kind of run the log and the report say. The trials themselves
    /// always run the core in scenario mode.
    pub mode: Mode,
    /// The wall-clock start, as `run_started` and the report say it.
    pub started_at: Option<String>,
    /// What each trial looks for besides the core's own notes.
    pub detect: Detect,
    /// The keys of `inputs`, for the report.
    pub inputs: Vec<String>,
    /// The price of the judge's tokens, for the cost in the report.
    pub price: Option<Price>,
}

/// Opens a fresh browser for a trial: a new context with the storage state.
/// The browser has gone to the start URL when it is returned, so the loop
/// can observe the first page at once.
pub trait Launch: Send {
    /// Opens the browser of the next trial, on the start page.
    fn launch(&mut self) -> Pending<'_, Result<Box<dyn Browser>, BrowserError>>;
}

/// Why the loop stopped starting trials.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stopped {
    /// Every trial was started.
    Completed,
    /// The time limit passed, or the window opens only at the limit or later.
    TimeUp,
    /// The user interrupted the run.
    Interrupted,
    /// The judge failed too many times in a row in the last trial.
    JudgeDown,
}

impl From<Stop> for Stopped {
    fn from(stop: Stop) -> Self {
        match stop {
            Stop::Completed => Self::Completed,
            Stop::TimeUp => Self::TimeUp,
            Stop::Interrupted => Self::Interrupted,
        }
    }
}

/// How an interview ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// The ledger and the report are written.
    Wrote {
        /// Why the loop stopped starting trials.
        stopped: Stopped,
        /// Whether a confirmed issue was found.
        confirmed: bool,
    },
    /// The reset command failed. The ledger of the trials before is written.
    ResetFailed(ResetError),
    /// A browser failed. The ledger of the trials before is written.
    Broke(BrowserError),
    /// The previous ledger's persona or purpose differs from this run's.
    Mismatch(Mismatch),
    /// A record failed.
    Failed(RunError),
    /// The user interrupted again while the records were being written.
    Abandoned,
}

/// Runs the trials of `plan`, then writes the ledger, the last line of the
/// event log and the report.
pub async fn interview(
    launch: &mut dyn Launch,
    judge: &mut dyn Judge,
    clock: &mut dyn Clock,
    store: &mut dyn Store,
    redactor: &Redactor,
    interrupt: &mut dyn Interrupt,
    plan: &Plan,
) -> Ending {
    let parts = course::Parts {
        launch,
        judge,
        clock,
        store,
        redactor,
    };
    course::conduct(parts, interrupt, plan).await
}
