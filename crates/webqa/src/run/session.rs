//! One whole run: the start of the log, the memory, the loop, and the
//! records written at the end.

#[cfg(test)]
mod tests;

use serde_json::Value;
use webqa_core::event::Event;
use webqa_core::explore::Memory;
use webqa_core::machine::{ExitReason, Outcome, Rules, State};
use webqa_core::ports::BrowserError;

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
    /// Where the memory comes from.
    pub start: Start,
}

/// Where the memory of a run comes from.
#[derive(Debug, Clone)]
pub enum Start {
    /// The stored memory of the origin, written back at the end.
    Stored,
    /// The memory a replay starts from; the stored one is left alone.
    Given(Memory),
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
    /// The browser failed, and the records so far are written.
    Broke(BrowserError),
    /// The user interrupted again while the records were being written.
    Abandoned,
    /// A record could not be written.
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
    let memory = match begin(ports, plan).await {
        Ok(memory) => memory,
        Err(error) => return Ending::Failed(error),
    };
    let started = ports.redactor.json(memory.to_json(&plan.origin));
    let explored = explore(ports, interrupt, rules, memory).await;
    close(ports, interrupt, explored, &started.into_inner(), plan).await
}

/// Writes the records of the loop that ended as `explored` says, from the
/// memory `started` the run started from.
async fn close(
    ports: &mut Ports<'_>,
    interrupt: &mut dyn Interrupt,
    explored: Result<Outcome, RunError>,
    started: &Value,
    plan: &Plan,
) -> Ending {
    match explored {
        Ok(outcome) => finish(ports, interrupt, outcome, started, plan).await,
        Err(RunError::Browser(error)) => broke(ports, interrupt, error, started, plan).await,
        Err(error) => Ending::Failed(error),
    }
}

/// Writes the records of the run that ended with `outcome`.
async fn finish(
    ports: &mut Ports<'_>,
    interrupt: &mut dyn Interrupt,
    outcome: Outcome,
    started: &Value,
    plan: &Plan,
) -> Ending {
    let ended = Ended {
        reason: outcome.reason,
        memory: Some(outcome.state.memory()),
        started,
    };
    match written(ports, interrupt, ended, plan).await {
        Ok(()) => Ending::finished(outcome, ports.record()),
        Err(ending) => ending,
    }
}

/// Writes the records of the run the browser failed in, from the state
/// before the failing step.
async fn broke(
    ports: &mut Ports<'_>,
    interrupt: &mut dyn Interrupt,
    error: BrowserError,
    started: &Value,
    plan: &Plan,
) -> Ending {
    let last = ports.last.take();
    let ended = Ended {
        reason: ExitReason::Error,
        memory: last.as_deref().map(State::memory),
        started,
    };
    match written(ports, interrupt, ended, plan).await {
        Ok(()) => Ending::Broke(error),
        Err(ending) => ending,
    }
}

/// What the records of a run that ended are written from.
struct Ended<'a> {
    /// Why it ended.
    reason: ExitReason,
    /// The memory to keep: `None` when no page was seen.
    memory: Option<&'a Memory>,
    /// The memory the run started from, as a memory file.
    started: &'a Value,
}

/// Writes the records of `ended`, unless the user interrupts again; the
/// ending when they are not all written.
async fn written(
    ports: &mut Ports<'_>,
    interrupt: &mut dyn Interrupt,
    ended: Ended<'_>,
    plan: &Plan,
) -> Result<(), Ending> {
    // The interruption that ended the run is spent; any other end can take one.
    let allowance = u8::from(ended.reason != ExitReason::UserAborted);
    let writing = write_records(ports, ended, plan);
    let done = guard(writing, interrupt, allowance).await;
    done.map_or(Err(Ending::Abandoned), |result| {
        result.map_err(Ending::Failed)
    })
}

/// Writes the first line and gets the memory: the one given, or the stored one.
async fn begin(ports: &mut Ports<'_>, plan: &Plan) -> Result<Memory, RunError> {
    let started = Event::RunStarted {
        schema: 1,
        mode: plan.meta.mode,
        started_at: plan.meta.started_at.clone(),
    };
    ports.log(started).await?;
    match &plan.start {
        Start::Given(given) => Ok(given.clone()),
        Start::Stored => stored(ports, &plan.origin).await,
    }
}

/// Reads the stored memory of `origin`, noting one that was reset.
async fn stored(ports: &mut Ports<'_>, origin: &str) -> Result<Memory, RunError> {
    let loaded = memory::load(ports.store(), origin).await?;
    if let Some(cause) = loaded.reset {
        ports.log(Event::MemoryReset { cause }).await?;
    }
    Ok(loaded.memory)
}

/// Writes the memory (unless one was given or none was kept), the last
/// line and the report, in this order, and nothing after a failure.
async fn write_records(
    ports: &mut Ports<'_>,
    ended: Ended<'_>,
    plan: &Plan,
) -> Result<(), RunError> {
    if let (Start::Stored, Some(memory)) = (&plan.start, ended.memory) {
        memory::save(ports.store, &plan.origin, memory, ports.redactor).await?;
    }
    ports.log(Event::finished(ended.reason)).await?;
    let (record, meta) = (&ports.record, &plan.meta);
    let report = report::build(record, ended.reason, meta, ended.started, ports.redactor);
    ports.store.write_report(&report).await?;
    Ok(())
}
