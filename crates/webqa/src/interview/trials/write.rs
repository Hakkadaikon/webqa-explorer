//! The records written at the end: the ledger, the last line of the log
//! and the report.

use webqa_core::config::purpose::Aim;
use webqa_core::event::{Event, Mode};
use webqa_core::interview::compare::{Mismatch, compare};
use webqa_core::interview::issue::{Record, Run, Seen};
use webqa_core::interview::ledger::{Goal, Head, Ledger, inferred_goal, write};
use webqa_core::interview::outcome::{Outcome, tally};
use webqa_core::machine::ExitReason;
use webqa_core::ports::{Clock, Judge, Store};
use webqa_core::redact::Redactor;

use super::record::{self, Trial};
use super::{Plan, Stopped};
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
    let compared = compare(
        ledger(closing.plan, &run),
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

/// The ledger of `run`: its head only.
fn ledger(plan: &Plan, run: &Run<'_>) -> Ledger {
    let outcomes: Vec<Outcome> = run.trials.iter().map(|trial| trial.outcome).collect();
    let interview = &plan.interview;
    Ledger {
        head: Head {
            persona: interview.persona.name.clone(),
            goal: goal(&interview.purpose.aim, run),
            start: plan.start.as_inner().clone(),
            finished: tally(&outcomes).finished,
            images: run.images,
        },
        entries: Vec::new(),
        unchecked_purposes: Vec::new(),
    }
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
    let Closing {
        clock,
        store,
        redactor,
        kept,
        trials,
        plan,
        ..
    } = closing;
    store.write_ledger(&write(ledger, redactor)).await?;
    let reason = reason(stopped, trials.last().map(|trial| trial.reason));
    let last = Event::finished(reason);
    let elapsed = clock.elapsed();
    kept.journal
        .write(None, last, elapsed, store, redactor)
        .await?;
    let meta = Meta {
        mode: Mode::Scenario,
        started_at: None,
        target: plan.start.clone(),
        inputs: Vec::new(),
        price: None,
    };
    store
        .write_report(&report::build(&kept.record, reason, &meta, redactor))
        .await?;
    Ok(Closed::Wrote {
        confirmed: kept.record.confirmed(),
    })
}

/// Why the report says the run ended: the time limit, the interruption, or
/// the last trial's end.
fn reason(stopped: Stopped, last: Option<ExitReason>) -> ExitReason {
    match stopped {
        Stopped::TimeUp => ExitReason::TimeExceeded,
        Stopped::Interrupted => ExitReason::UserAborted,
        Stopped::Completed => last.unwrap_or(ExitReason::UserAborted),
    }
}
