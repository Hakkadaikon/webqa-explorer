//! Tests of interrupting the run.

use std::collections::BTreeSet;
use std::future;
use std::mem;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;
use webqa_core::effect::Touch;
use webqa_core::judge::Image;
use webqa_core::machine::{ExitReason, Facts, Heard};
use webqa_core::ports::{Browser, BrowserError, Observation, Pending};

use super::super::fakes::FakeBrowser;
use super::super::*;
use super::{Fakes, Never, rules, save};

/// Records the touch, lets the gate through, then waits for the page forever.
struct StuckBrowser {
    gate: Arc<Notify>,
    inner: FakeBrowser,
}

impl Browser for StuckBrowser {
    fn observe(&mut self) -> Pending<'_, Result<Observation, BrowserError>> {
        self.inner.observe()
    }

    fn touch<'a>(&'a mut self, touch: &'a Touch) -> Pending<'a, Result<(), BrowserError>> {
        self.inner.touches.push(touch.clone());
        self.gate.notify_one();
        Box::pin(future::pending())
    }

    fn screenshot(&mut self) -> Pending<'_, Result<Image, BrowserError>> {
        self.inner.screenshot()
    }
}

/// Interrupts once the gate is let through.
struct GateInterrupt(Arc<Notify>);

impl Interrupt for GateInterrupt {
    fn wait(&mut self) -> Pending<'_, ()> {
        Box::pin(Arc::clone(&self.0).notified_owned())
    }
}

/// Interrupts on the calls of `wait` numbered in `fire_on`, from 1.
struct CountInterrupt {
    calls: u32,
    fire_on: BTreeSet<u32>,
}

impl Interrupt for CountInterrupt {
    fn wait(&mut self) -> Pending<'_, ()> {
        self.calls += 1;
        if self.fire_on.contains(&self.calls) {
            Box::pin(future::ready(()))
        } else {
            Box::pin(future::pending())
        }
    }
}

/// The run of [`Fakes::new`], interrupted while the browser touches `保存`.
async fn interrupted_mid_touch() -> (Fakes, Outcome) {
    let mut fakes = Fakes::new();
    let gate = Arc::new(Notify::new());
    let inner = mem::take(&mut fakes.browser);
    let mut browser = StuckBrowser {
        gate: Arc::clone(&gate),
        inner,
    };
    let mut ports = Ports::new(
        &mut browser,
        &mut fakes.judge,
        &mut fakes.clock,
        &mut fakes.store,
        &fakes.redactor,
    );
    let mut interrupt = GateInterrupt(gate);
    let outcome = explore(&mut ports, &mut interrupt, rules(), Memory::default());
    let outcome = outcome.await.unwrap();
    fakes.browser = browser.inner;
    (fakes, outcome)
}

#[tokio::test]
async fn interrupt_during_browser_op_skips_page_wait() {
    let (fakes, outcome) = interrupted_mid_touch().await;
    assert_eq!(
        outcome.reason,
        ExitReason::UserAborted,
        "the user interrupted"
    );
    let touch = Touch {
        op: "click".to_owned(),
        target: Some(save()),
        confidence: 0.9,
    };
    assert_eq!(fakes.browser.touches, [touch], "the touch had started");
    assert_eq!(
        fakes.clock.now,
        Duration::from_millis(1500),
        "no waiting for the page"
    );
    let start = outcome.state.origin().unwrap();
    let operated = outcome.state.memory().is_operated(start, &save());
    assert!(!operated, "the touch is not taken as done");
    assert_eq!(fakes.kinds(), ["asked", "decided"], "no result line");
    interrupt_before_the_touch_starts_nothing().await;
}

/// The third `wait` belongs to the touch: the question and the second look
/// each wait once.
async fn interrupt_before_the_touch_starts_nothing() {
    let mut fakes = Fakes::new();
    let mut interrupt = CountInterrupt {
        calls: 0,
        fire_on: BTreeSet::from([3]),
    };
    let outcome = explore(
        &mut fakes.ports(),
        &mut interrupt,
        rules(),
        Memory::default(),
    )
    .await;
    assert_eq!(
        outcome.map(|o| o.reason),
        Ok(ExitReason::UserAborted),
        "interrupted"
    );
    assert_eq!(fakes.browser.touches, Vec::new(), "the touch never started");
    assert_eq!(interrupt.calls, 3, "one wait for each effect");
}

#[tokio::test]
async fn interrupted_run_changes_nothing() {
    let (mut fakes, outcome) = interrupted_mid_touch().await;
    let state = (*outcome.state).clone();
    let again = machine::step(
        state,
        Facts::Outside(Heard::Woke),
        Duration::from_secs(3600),
    );
    assert_eq!(
        again.reason(),
        Some(ExitReason::UserAborted),
        "still interrupted"
    );
    let counted = counts(again.state());
    assert_eq!(counted, counts(&outcome.state), "nothing counted");
    let before = effects(&fakes);
    let ran = run(Step::Finished(outcome), &mut fakes.ports(), &mut Never).await;
    let ended = Ok(ExitReason::UserAborted);
    assert_eq!(ran.map(|o| o.reason), ended, "the same reason");
    assert_eq!(effects(&fakes), before, "nothing asked, touched or written");
}

/// The decisions used and the screens remembered.
const fn counts(state: &State) -> (u32, usize) {
    (state.budget().used(), state.memory().screen_count())
}

/// What the fakes were asked to do so far.
fn effects(f: &Fakes) -> (Vec<String>, Vec<Touch>, Vec<serde_json::Value>, usize) {
    let touches = f.browser.touches.clone();
    let lines = f.store.lines.clone();
    (f.judge.texts.clone(), touches, lines, f.store.writes)
}
