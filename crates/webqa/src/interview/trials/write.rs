//! The records written at the end: the ledger, the last line of the log
//! and the report.

use serde_json::Value;

use webqa_core::config::purpose::Aim;
use webqa_core::event::Event;
use webqa_core::interview::compare::{Mismatch, compare};
use webqa_core::interview::issue::{Record, Run, Seen};
use webqa_core::interview::ledger::{Goal, Head, Ledger, inferred_goal, write};
use webqa_core::interview::outcome::{Outcome, tally};
use webqa_core::machine::{ExitReason, StuckKind};
use webqa_core::ports::{BrowserError, Clock, Judge, Store};
use webqa_core::redact::Redactor;

use super::after::{Asker, Findings, findings};
use super::record::{self, Trial};
use super::shots::write_shots;
use super::{Ending, Plan, Stopped};
use crate::interview::reset::ResetError;
use crate::run::{Kept, RunError};
use crate::store::report::{self, Meta};

/// What the records are written from.
pub(super) struct Closing<'a> {
    pub(super) judge: &'a mut dyn Judge,
    pub(super) clock: &'a mut dyn Clock,
    pub(super) store: &'a mut dyn Store,
    pub(super) redactor: &'a Redactor,
    pub(super) kept: &'a mut Kept,
    pub(super) trials: &'a [Trial],
    pub(super) plan: &'a Plan,
    /// The failure that stopped the loop, if one did.
    pub(super) failed: Option<Failure>,
}

/// What stopped the loop before its trials were done.
#[derive(Debug, Clone, Copy)]
pub(super) enum Failure {
    /// The reset command failed.
    Reset(ResetError),
    /// A browser failed.
    Browser(BrowserError),
}

impl Failure {
    /// How the run ends once the records of the trials before are written.
    pub(super) const fn ending(self) -> Ending {
        match self {
            Self::Reset(error) => Ending::ResetFailed(error),
            Self::Browser(error) => Ending::Broke(error),
        }
    }
}

/// What came of writing the records.
pub(super) enum Closed {
    /// The ledger, the last line and the report are written.
    Wrote {
        /// Whether a confirmed issue was found.
        confirmed: bool,
    },
    /// The previous ledger is of another persona or purpose; nothing is
    /// written.
    Mismatch(Mismatch),
}

/// Writes the ledger, the last line of the log and the report of a run
/// that stopped for `stopped`.
pub(super) async fn records(closing: Closing<'_>, stopped: Stopped) -> Result<Closed, RunError> {
    let views: Vec<Vec<Seen<'_>>> = closing.trials.iter().map(views).collect();
    let records: Vec<Record<'_>> = closing.trials.iter().zip(&views).map(record).collect();
    let run = Run {
        trials: &records,
        images: closing.judge.reads_images(),
    };
    let mut asker = Asker {
        judge: &mut *closing.judge,
        clock: &mut *closing.clock,
        done: &mut closing.kept.record,
    };
    let found = findings(&mut asker, closing.plan, &run).await;
    let compared = compare(
        ledger(closing.plan, &run, found),
        closing.plan.previous.as_ref(),
        &run,
    );
    match compared {
        Err(mismatch) => Ok(Closed::Mismatch(mismatch)),
        Ok(ledger) => put(closing, &ledger, stopped).await,
    }
}

/// The steps of `trial` as the issues read them.
fn views(trial: &Trial) -> Vec<Seen<'_>> {
    trial.steps.iter().map(seen).collect()
}

/// `step` as the issues read it.
fn seen(step: &record::Step) -> Seen<'_> {
    Seen {
        step: step.logged,
        screen: &step.screen,
        viewport: step.viewport,
        reply: step.reply.as_ref(),
        clues: &step.clues,
        ran: step.ran,
    }
}

/// `trial` with its steps `views`, as the issues read it.
fn record<'a>((trial, views): (&Trial, &'a Vec<Seen<'a>>)) -> Record<'a> {
    Record {
        outcome: trial.outcome,
        steps: views,
    }
}

/// The ledger of `run` with what the questions after the trials found.
fn ledger(plan: &Plan, run: &Run<'_>, found: Findings) -> Ledger {
    let outcomes: Vec<Outcome> = run.trials.iter().map(|trial| trial.outcome).collect();
    let interview = &plan.interview;
    let mut ledger = Ledger {
        head: Head {
            persona: interview.persona.name.clone(),
            goal: goal(&interview.purpose.aim, run),
            start: plan.start.as_inner().clone(),
            finished: tally(&outcomes).finished,
            images: run.images,
        },
        entries: found.entries,
        unchecked_purposes: found.unchecked,
    };
    ledger.arrange();
    ledger
}

/// The purpose of the run: the written one, or what the judge guessed.
fn goal(aim: &Aim, run: &Run<'_>) -> Goal {
    match aim {
        Aim::Written(written) => Goal::Written(written.goal.clone()),
        Aim::Inferred => Goal::Inferred(inferred_goal(run)),
    }
}

/// Writes `ledger`, the last line and the report.
async fn put(closing: Closing<'_>, ledger: &Ledger, stopped: Stopped) -> Result<Closed, RunError> {
    write_shots(&mut *closing.store, closing.trials, ledger).await?;
    let text = write(ledger, closing.redactor);
    closing.store.write_ledger(&text).await?;
    let why = reason_of(&closing, stopped);
    let confirmed = conclude(closing, why).await?;
    Ok(Closed::Wrote { confirmed })
}

/// Writes the last line of the log and the report of a run that ended for
/// `reason`. Whether a confirmed issue was found.
async fn conclude(closing: Closing<'_>, reason: ExitReason) -> Result<bool, RunError> {
    let Closing {
        clock,
        store,
        redactor,
        kept,
        plan,
        ..
    } = closing;
    let last = Event::finished(reason);
    let journal = &mut kept.journal;
    journal
        .write(None, last, clock.elapsed(), store, redactor)
        .await?;
    let meta = Meta {
        mode: plan.mode,
        started_at: plan.started_at.clone(),
        target: plan.start.clone(),
        inputs: plan.inputs.clone(),
        price: plan.price,
    };
    let report = report::build(&kept.record, reason, &meta, &Value::Null, redactor);
    store.write_report(&report).await?;
    Ok(kept.record.confirmed())
}

/// Why the report says the run of `closing` ended after it stopped for
/// `stopped`.
fn reason_of(closing: &Closing<'_>, stopped: Stopped) -> ExitReason {
    let last = closing.trials.last().map(|trial| trial.reason);
    match closing.failed {
        Some(Failure::Browser(_)) => ExitReason::Error,
        Some(Failure::Reset(_)) => reason(stopped, closing.trials.is_empty(), last),
        None => reason(stopped, false, last),
    }
}

/// What the report says of a run whose reset command failed before any
/// trial: the run got nowhere. No reason of the core is nearer, as none is a
/// failure. A failure after a trial leaves the reason of the last trial.
const RESET_FAILED: ExitReason = ExitReason::Stuck(StuckKind::NoProgress);

/// Why the report says the run ended: the reset command's failure before any
/// trial, the time limit, the interruption, or the last trial's end.
fn reason(stopped: Stopped, nowhere: bool, last: Option<ExitReason>) -> ExitReason {
    match (nowhere, stopped) {
        (true, _) => RESET_FAILED,
        (false, Stopped::TimeUp) => ExitReason::TimeExceeded,
        (false, Stopped::Interrupted) => ExitReason::UserAborted,
        (false, Stopped::Completed) => last.unwrap_or(ExitReason::UserAborted),
    }
}
