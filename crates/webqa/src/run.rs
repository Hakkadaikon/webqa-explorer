//! The run loop: does the effects the core asks for and passes back the facts.

mod detect;
#[cfg(test)]
#[path = "../tests/support/fakes.rs"]
pub(crate) mod fakes;
mod interviewing;
mod journal;
mod log;
mod notes;
mod perform;
pub mod session;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use webqa_core::effect::Effect;
use webqa_core::event::Event;
use webqa_core::explore::Memory;
use webqa_core::issue::Probe;
use webqa_core::machine::{self, Outcome, Rules, State, Step};
use webqa_core::ports::{Browser, BrowserError, Clock, Judge, Store, StoreError};
use webqa_core::redact::Redactor;
use webqa_core::window::RequestWindow;

use crate::interrupt::Interrupt;
use crate::record::Record;

pub use interviewing::{Interviewing, Taken};
pub use journal::Journal;

/// Why the run stopped before an outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RunError {
    /// The browser failed.
    #[error(transparent)]
    Browser(#[from] BrowserError),
    /// A record could not be written.
    #[error(transparent)]
    Store(#[from] StoreError),
}

/// What the run loop looks for besides the core's own notes.
#[derive(Debug, Clone, PartialEq)]
pub struct Detect {
    /// `limits.error_confidence`: the error display and the semantic
    /// findings count from this probability.
    pub error_confidence: f64,
    /// The semantic findings to ask about each step. Empty asks none.
    pub probes: Vec<Probe>,
}

impl Default for Detect {
    fn default() -> Self {
        Self {
            error_confidence: 0.8,
            probes: Vec::new(),
        }
    }
}

/// The ports of a run, where it is in the event log, and what it did so far.
pub struct Ports<'a> {
    browser: &'a mut dyn Browser,
    judge: &'a mut dyn Judge,
    clock: &'a mut dyn Clock,
    store: &'a mut dyn Store,
    redactor: &'a Redactor,
    /// The number of the next line of the log, and whether a write failed.
    journal: Journal,
    /// The current step: 0 until the first question or core operation.
    step: u64,
    detect: Detect,
    record: Record,
    /// The names of the screenshots written.
    shots: BTreeSet<String>,
    /// The interview's questions in place of the core's: `None` for the
    /// other modes.
    interview: Option<Interviewing>,
    /// The state before the step the browser failed in: `None` until then,
    /// and when it failed on the first page.
    last: Option<Box<State>>,
}

/// What a run keeps from one browser to the next: the log's counters, what
/// the run did and the screenshots written.
#[derive(Debug, Default)]
pub struct Kept {
    /// The log's counters.
    pub journal: Journal,
    /// What the run did so far.
    pub record: Record,
    /// The names of the screenshots written.
    pub shots: BTreeSet<String>,
}

impl<'a> Ports<'a> {
    /// Gathers the ports. Every page and line passes through `redactor`.
    /// The run looks for issues as [`Detect::default`] says.
    pub fn new(
        browser: &'a mut dyn Browser,
        judge: &'a mut dyn Judge,
        clock: &'a mut dyn Clock,
        store: &'a mut dyn Store,
        redactor: &'a Redactor,
    ) -> Self {
        Self {
            browser,
            judge,
            clock,
            store,
            redactor,
            journal: Journal::default(),
            step: 0,
            detect: Detect::default(),
            record: Record::default(),
            shots: BTreeSet::new(),
            interview: None,
            last: None,
        }
    }

    /// The same ports, asking the interview's questions in place of the
    /// core's.
    #[must_use]
    pub fn interviewing(self, interview: Interviewing) -> Self {
        Self {
            interview: Some(interview),
            ..self
        }
    }

    /// The steps of the interview that asked the judge, which are no longer
    /// kept. Empty outside the interview.
    pub fn take_steps(&mut self) -> Vec<Taken> {
        let interview = self.interview.as_mut();
        interview.map(Interviewing::take).unwrap_or_default()
    }

    /// The same ports, going on from what an earlier run with another
    /// browser kept.
    #[must_use]
    pub fn resumed(self, kept: Kept) -> Self {
        Self {
            journal: kept.journal,
            record: kept.record,
            shots: kept.shots,
            ..self
        }
    }

    /// What the next browser's ports keep going from.
    #[must_use]
    pub fn into_kept(self) -> Kept {
        Kept {
            journal: self.journal,
            record: self.record,
            shots: self.shots,
        }
    }

    /// The same ports, looking for issues as `detect` says.
    #[must_use]
    pub fn detecting(self, detect: Detect) -> Self {
        Self { detect, ..self }
    }

    /// What the run did so far.
    #[must_use]
    pub const fn record(&self) -> &Record {
        &self.record
    }

    /// The store, for the records written around the run.
    pub fn store(&mut self) -> &mut dyn Store {
        self.store
    }

    /// The redactor every record passes through.
    #[must_use]
    pub const fn redactor(&self) -> &Redactor {
        self.redactor
    }

    /// Writes `event` as the next line of the log, with the current step.
    ///
    /// # Errors
    ///
    /// [`RunError::Store`] when the line cannot be written.
    pub async fn log(&mut self, event: Event) -> Result<(), RunError> {
        let step = Some(self.step).filter(|&step| step > 0);
        let elapsed = self.clock.elapsed();
        let store = &mut *self.store;
        self.journal
            .write(step, event, elapsed, store, self.redactor)
            .await
    }

    /// `written`, remembering a failure so that nothing is written after it.
    pub(super) fn kept(&mut self, written: Result<(), StoreError>) -> Result<(), RunError> {
        self.journal.kept(written)
    }
}

/// Observes the page, then runs from there under `rules`, exploring with
/// `memory`, until the core finishes or the user interrupts.
///
/// # Errors
///
/// [`RunError`] when the browser fails or a record cannot be written.
pub async fn explore(
    ports: &mut Ports<'_>,
    interrupt: &mut dyn Interrupt,
    rules: Rules,
    memory: Memory,
) -> Result<Outcome, RunError> {
    let window = RequestWindow::new(rules.requests_per_hour);
    explore_in(ports, interrupt, rules, memory, window).await
}

/// As [`explore`], with the core's window `window` instead of a new one.
///
/// # Errors
///
/// As [`explore`].
pub async fn explore_in(
    ports: &mut Ports<'_>,
    interrupt: &mut dyn Interrupt,
    rules: Rules,
    memory: Memory,
    window: RequestWindow,
) -> Result<Outcome, RunError> {
    let seen = perform::observe(ports).await?;
    let start = machine::start_in(seen, rules, memory, window, ports.clock.elapsed());
    notes::after((0, 0), &start, ports).await?;
    run(start, ports, interrupt).await
}

/// Does effects until the core finishes or the user interrupts.
///
/// # Errors
///
/// As [`explore`].
pub async fn run(
    start: Step,
    ports: &mut Ports<'_>,
    interrupt: &mut dyn Interrupt,
) -> Result<Outcome, RunError> {
    let mut step = start;
    loop {
        match step {
            Step::Continue(state, effect) => {
                let kept = (*state).clone();
                tokio::select! {
                    biased;
                    () = interrupt.wait() => return Ok(machine::abort(kept)),
                    next = advance(*state, effect, ports) => {
                        step = next.inspect_err(|_| ports.last = Some(Box::new(kept)))?;
                    }
                }
            }
            Step::Finished(outcome) => return Ok(outcome),
        }
    }
}

/// Does one effect, passes its facts to the core and logs what the core noted.
async fn advance(state: State, effect: Effect, ports: &mut Ports<'_>) -> Result<Step, RunError> {
    let before = (state.notes().len(), state.stage());
    let facts = perform::perform(effect, &state, ports).await?;
    let next = machine::step(state, facts, ports.clock.elapsed());
    notes::after(before, &next, ports).await?;
    Ok(next)
}
