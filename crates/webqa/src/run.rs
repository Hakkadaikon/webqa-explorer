//! The run loop: does the effects the core asks for and passes back the facts.

#[cfg(test)]
#[path = "../tests/support/fakes.rs"]
mod fakes;
#[cfg(test)]
mod tests;

use std::time::Duration;

use webqa_core::effect::{Effect, Outside, PageEffect, Touch};
use webqa_core::event::{Event, Line};
use webqa_core::machine::{self, Facts, Heard, Outcome, Seen, State, Step};
use webqa_core::ports::{Browser, BrowserError, Clock, Judge, Store, StoreError};
use webqa_core::redact::Redactor;
use webqa_core::screen;

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

/// The ports of a run and where it is in the event log.
pub struct Ports<'a> {
    browser: &'a mut dyn Browser,
    judge: &'a mut dyn Judge,
    clock: &'a mut dyn Clock,
    store: &'a mut dyn Store,
    redactor: &'a Redactor,
    seq: u64,
}

impl<'a> Ports<'a> {
    /// Gathers the ports. Every page and line passes through `redactor`.
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
            seq: 0,
        }
    }
}

/// Observes the page, then runs from there, choosing among the operations `ops`.
///
/// # Errors
///
/// [`RunError`] when the browser fails or a record cannot be written.
pub async fn explore(ports: &mut Ports<'_>, ops: Vec<String>) -> Result<Outcome, RunError> {
    let seen = observe(ports).await?;
    run(machine::start(seen, ops), ports).await
}

/// Does effects until the core finishes.
///
/// # Errors
///
/// [`RunError`] when the browser fails or a record cannot be written.
pub async fn run(start: Step, ports: &mut Ports<'_>) -> Result<Outcome, RunError> {
    let mut step = start;
    loop {
        match step {
            Step::Continue(state, effect) => step = advance(state, effect, ports).await?,
            Step::Finished(outcome) => return Ok(outcome),
        }
    }
}

async fn advance(state: State, effect: Effect, ports: &mut Ports<'_>) -> Result<Step, RunError> {
    let facts = perform(effect, state.last(), ports).await?;
    Ok(machine::step(state, facts))
}

/// Does the effect. `last` is the page before it.
async fn perform(effect: Effect, last: &Seen, ports: &mut Ports<'_>) -> Result<Facts, RunError> {
    match effect {
        Effect::Outside(outside) => Ok(Facts::Outside(beyond(outside, ports).await)),
        Effect::Page(page) => on_page(page, last, ports).await,
    }
}

async fn beyond(outside: Outside, ports: &mut Ports<'_>) -> Heard {
    match outside {
        Outside::Ask(questions) => Heard::Answered(
            ports
                .judge
                .ask(&questions.screen, &questions.questions)
                .await,
        ),
        Outside::Sleep(wake) => {
            ports.clock.sleep_until(wake.at).await;
            Heard::Woke
        }
    }
}

async fn on_page(page: PageEffect, last: &Seen, ports: &mut Ports<'_>) -> Result<Facts, RunError> {
    match page {
        PageEffect::Look(_) => observe(ports).await.map(Facts::Page),
        PageEffect::Touch(touch) => touched(&touch, last, ports).await.map(Facts::Page),
    }
}

/// Logs the decision, touches the page and logs what came of it.
async fn touched(touch: &Touch, before: &Seen, ports: &mut Ports<'_>) -> Result<Seen, RunError> {
    record(ports, decided(touch)).await?;
    ports.browser.touch(touch).await?;
    executed(before, ports).await
}

fn decided(touch: &Touch) -> Event {
    Event::Decided {
        op: touch.op.clone(),
        target: touch.target.as_ref().map(|id| id.name.clone()),
        p: touch.confidence,
    }
}

/// Observes the page after the touch and logs whether it changed.
///
/// Provisional: `accesses` is always 1, one access per touch, until the
/// accesses are counted with the time window.
async fn executed(before: &Seen, ports: &mut Ports<'_>) -> Result<Seen, RunError> {
    let after = observe(ports).await?;
    let changed = screen::changed(&before.screen, &after.screen, after.viewport);
    record(
        ports,
        Event::Executed {
            changed,
            accesses: 1,
        },
    )
    .await?;
    Ok(after)
}

/// Reads the page and redacts it.
async fn observe(ports: &mut Ports<'_>) -> Result<Seen, RunError> {
    let observation = ports.browser.observe().await?;
    Ok(Seen {
        screen: ports.redactor.screen(observation.screen),
        viewport: observation.viewport,
    })
}

/// Writes the event as the next line of the log.
///
/// Provisional: `step` is always 1, since the run has one step until the
/// state machine counts its steps.
async fn record(ports: &mut Ports<'_>, event: Event) -> Result<(), RunError> {
    let line = Line {
        seq: ports.seq,
        elapsed_ms: millis(ports.clock.elapsed()),
        step: Some(1),
        event,
    };
    ports.seq += 1;
    // Every field of a line serializes to JSON, so this never falls back.
    let line = ports
        .redactor
        .json(serde_json::to_value(&line).unwrap_or_default());
    ports
        .store
        .write_event(&line)
        .await
        .map_err(RunError::Store)
}

fn millis(elapsed: Duration) -> u64 {
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}
