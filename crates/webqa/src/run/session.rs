//! One whole run: the start of the log, the memory, the loop, and the
//! records written at the end.

#[cfg(test)]
mod tests;

use webqa_core::event::Event;
use webqa_core::explore::Memory;
use webqa_core::machine::{ExitReason, Outcome, Rules};

use super::{Ports, RunError, explore};
use crate::interrupt::{Interrupt, guard};
use crate::record::Record;
use crate::store::report::Meta;
use crate::store::{memory, report};

/// What a run needs besides its ports and rules.
#[derive(Debug, Clone)]
pub struct Plan {
    /// The key of the memory: the origin of `target.url`.
    pub origin: String,
    /// What the report says besides the record.
    pub meta: Meta,
}

/// How a run ended.
#[derive(Debug)]
pub enum Ending {
    /// The core or the user ended it, and the records are written.
    Finished {
        /// Why it ended.
        outcome: Outcome,
        /// Whether a confirmed issue was found.
        confirmed: bool,
    },
    /// The user interrupted again while the records were being written.
    Abandoned,
    /// The browser failed or a record could not be written.
    Failed(RunError),
}

impl Ending {
    /// The run ended with `outcome`; `record` tells whether a confirmed
    /// issue was found.
    #[must_use]
    pub fn finished(outcome: Outcome, record: &Record) -> Self {
        Self::Finished {
            outcome,
            confirmed: record.confirmed(),
        }
    }
}

/// Runs once: starts the log, reads the memory, runs the loop, then writes
/// the memory, the last line and the report, watching for interruptions.
pub async fn conduct(
    ports: &mut Ports<'_>,
    interrupt: &mut dyn Interrupt,
    rules: Rules,
    plan: &Plan,
) -> Ending {
    match run_loop(ports, interrupt, rules, plan).await {
        Ok(outcome) => finish(ports, interrupt, outcome, plan).await,
        Err(error) => Ending::Failed(error),
    }
}

/// Writes the records of the run that ended with `outcome`.
async fn finish(
    ports: &mut Ports<'_>,
    interrupt: &mut dyn Interrupt,
    outcome: Outcome,
    plan: &Plan,
) -> Ending {
    // The interruption that ended the run is spent; any other end can take one.
    let allowance = u8::from(outcome.reason != ExitReason::UserAborted);
    let written = guard(write_records(ports, &outcome, plan), interrupt, allowance);
    match written.await {
        None => Ending::Abandoned,
        Some(Err(error)) => Ending::Failed(error),
        Some(Ok(())) => Ending::finished(outcome, ports.record()),
    }
}

/// Starts the log and the memory, then runs the loop.
async fn run_loop(
    ports: &mut Ports<'_>,
    interrupt: &mut dyn Interrupt,
    rules: Rules,
    plan: &Plan,
) -> Result<Outcome, RunError> {
    let memory = begin(ports, plan).await?;
    explore(ports, interrupt, rules, memory).await
}

/// Writes the first line and reads the memory, noting one that was reset.
async fn begin(ports: &mut Ports<'_>, plan: &Plan) -> Result<Memory, RunError> {
    let started = Event::RunStarted {
        schema: 1,
        mode: plan.meta.mode,
        started_at: plan.meta.started_at.clone(),
    };
    ports.log(started).await?;
    let loaded = memory::load(ports.store(), &plan.origin).await?;
    if let Some(cause) = loaded.reset {
        ports.log(Event::MemoryReset { cause }).await?;
    }
    Ok(loaded.memory)
}

/// Writes the memory, the last line and the report, in this order, and
/// nothing after a failure.
async fn write_records(
    ports: &mut Ports<'_>,
    outcome: &Outcome,
    plan: &Plan,
) -> Result<(), RunError> {
    let origin = &plan.origin;
    memory::save(ports.store, origin, outcome.state.memory(), ports.redactor).await?;
    ports.log(Event::finished(outcome.reason)).await?;
    let report = report::build(&ports.record, outcome.reason, &plan.meta, ports.redactor);
    ports.store.write_report(&report).await?;
    Ok(())
}
