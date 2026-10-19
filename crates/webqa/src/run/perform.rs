//! One effect at a time: asking the judge, waiting, looking at and
//! operating the page.

use std::time::Duration;

use webqa_core::effect::{Effect, Outside, PageEffect, Questions, Touch};
use webqa_core::event::Event;
use webqa_core::issue::{answered_issues, probe_questions};
use webqa_core::machine::{Cause, Facts, Heard, Material, Seen, State, Whole};
use webqa_core::ports::Observation;
use webqa_core::redact::{Redacted, Redactor};
use webqa_core::screen::{self, Screen};

use super::detect::{note_issues, observed_issues};
use super::{Ports, RunError, log};
use crate::record::{Act, Call};

/// The name of the operation that types into an input field.
const INPUT: &str = "input";

/// Does the effect the core asked for in `state`.
pub(super) async fn perform(
    effect: Effect,
    state: &State,
    ports: &mut Ports<'_>,
) -> Result<Facts, RunError> {
    match effect {
        Effect::Outside(outside) => beyond(outside, ports).await.map(Facts::Outside),
        Effect::Page(page) => on_page(page, state, ports).await.map(page_facts),
    }
}

async fn beyond(outside: Outside, ports: &mut Ports<'_>) -> Result<Heard, RunError> {
    match outside {
        Outside::Ask(questions) => ask(questions, ports).await,
        Outside::Sleep(wake) => {
            ports.log(log::waited(wake.at)).await?;
            ports.clock.sleep_until(wake.at).await;
            Ok(Heard::Woke)
        }
    }
}

/// Asks the judge, the semantic findings after the core's questions, and
/// records the call and the issues its answers show.
async fn ask(questions: Questions, ports: &mut Ports<'_>) -> Result<Heard, RunError> {
    ports.step += 1;
    let mut asking = questions.questions;
    asking.extend(probe_questions(&ports.detect.probes));
    let started = ports.clock.elapsed();
    let reply = ports.judge.ask(&questions.screen, &asking).await;
    let think = ports.clock.elapsed().saturating_sub(started);
    ports.log(log::asked(&reply, think)).await?;
    ports.record.calls.push(Call {
        step: ports.step,
        reply: reply.clone(),
        think,
    });
    let detect = &ports.detect;
    let issues = reply.as_ref().map_or_else(
        |_| Vec::new(),
        |answers| answered_issues(answers, detect.error_confidence, &detect.probes),
    );
    note_issues(ports, issues).await?;
    Ok(Heard::Answered(reply))
}

async fn on_page(page: PageEffect, state: &State, ports: &mut Ports<'_>) -> Result<Seen, RunError> {
    match page {
        PageEffect::Look(_) => observe(ports).await,
        PageEffect::Touch(touch) => touched(&touch, state, ports).await,
    }
}

/// The facts of observing `seen`.
fn page_facts(seen: Seen) -> Facts {
    Facts::Page(Box::new(seen))
}

/// Logs who asked for the touch, touches the page, observes it, and
/// records the operation with its timings.
async fn touched(touch: &Touch, state: &State, ports: &mut Ports<'_>) -> Result<Seen, RunError> {
    // A state that asks for a touch always knows who asked; the default
    // only keeps the type total.
    let cause = state.cause().unwrap_or(Cause::Judge);
    announce(touch, cause, state, ports).await?;
    let (after, timings) = act(touch, ports).await?;
    let changed = screen::changed(&state.last().screen, &after.screen, after.viewport);
    ports
        .log(Event::Executed {
            changed,
            accesses: 1,
        })
        .await?;
    ports.record.acts.push(Act {
        step: ports.step,
        op: touch.op.clone(),
        target: touch.target.clone(),
        p: probability(touch, cause),
        cause,
        act: timings.0,
        settle: timings.1,
    });
    Ok(after)
}

/// Logs the judge's decision, or a new step for the core's own operation,
/// and whether the browser will do nothing for it.
async fn announce(
    touch: &Touch,
    cause: Cause,
    state: &State,
    ports: &mut Ports<'_>,
) -> Result<(), RunError> {
    let event = asker(touch, cause, &mut ports.step);
    ports.log(event).await?;
    let skip = state
        .rules()
        .input_skip(&state.last().screen, touch.target.as_ref());
    if let Some(why) = skip.filter(|_| touch.op == INPUT) {
        ports.log(log::skipped(why)).await?;
    }
    Ok(())
}

/// The line of who asked for `touch`. The core's own operation starts a
/// new step.
fn asker(touch: &Touch, cause: Cause, step: &mut u64) -> Event {
    if cause == Cause::Judge {
        log::decided(touch)
    } else {
        *step += 1;
        log::recovered(touch, cause)
    }
}

/// The judge's probability for its own operations, 1 for the core's.
fn probability(touch: &Touch, cause: Cause) -> f64 {
    if cause == Cause::Judge {
        touch.confidence
    } else {
        1.0
    }
}

/// Touches the page and observes it: the page after, how long the touch
/// took and how long the page took to be observed.
async fn act(
    touch: &Touch,
    ports: &mut Ports<'_>,
) -> Result<(Seen, (Duration, Duration)), RunError> {
    let started = ports.clock.elapsed();
    ports.browser.touch(touch).await?;
    let acted = ports.clock.elapsed();
    let after = observe(ports).await?;
    let settled = ports.clock.elapsed();
    let timings = (acted.saturating_sub(started), settled.saturating_sub(acted));
    Ok((after, timings))
}

/// Reads the page and redacts it, keeping the material from before
/// redaction, then records the issues it shows.
pub(super) async fn observe(ports: &mut Ports<'_>) -> Result<Seen, RunError> {
    let observation = ports.browser.observe().await?;
    let redactor = ports.redactor;
    let console: Vec<Redacted<String>> = observation
        .console
        .iter()
        .map(|error| redactor.text(error))
        .collect();
    let seen = seen(observation, redactor);
    note_issues(ports, observed_issues(&seen, &console)).await?;
    Ok(seen)
}

/// The redacted page of `observation`. The whole page is redacted too, its
/// elements under the page's URL.
fn seen(observation: Observation, redactor: &Redactor) -> Seen {
    let outline = Screen {
        url: observation.screen.url.clone(),
        elements: observation.outline,
    };
    Seen {
        whole: Whole {
            outline: redactor.screen(outline),
            text: redactor.text(&observation.text),
        },
        screen: redactor.screen(observation.screen),
        viewport: observation.viewport,
        material: Material {
            paywall: observation.paywall,
            candidates: observation.candidates,
            nodes: observation.nodes,
        },
    }
}
