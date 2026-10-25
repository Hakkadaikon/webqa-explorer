//! Tests of the timings the run records and of waiting on the clock.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::{future, mem};

use webqa_core::config::raw::Op;
use webqa_core::effect::Touch;
use webqa_core::judge::{Answers, Image, JudgeError, Question};
use webqa_core::machine::{Cause, ExitReason};
use webqa_core::ports::{Browser, BrowserError, Clock, Judge, Observation, Pending};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;

use super::super::fakes::{FakeBrowser, RecordingJudge};
use super::super::*;
use super::{Fakes, limited, page, rules, run_on, script};

/// Milliseconds since the run started, shared by the fakes that take time.
#[derive(Debug, Clone, Default)]
struct Shared(Arc<AtomicU64>);

impl Shared {
    fn pass(&self, ms: u64) {
        self.0.fetch_add(ms, Ordering::SeqCst);
    }
}

/// Reads the shared time, and sleeps by setting it.
struct TickingClock(Shared);

impl Clock for TickingClock {
    fn elapsed(&self) -> Duration {
        Duration::from_millis(self.0.0.load(Ordering::SeqCst))
    }

    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        self.0.0.store(log::millis(at), Ordering::SeqCst);
        Box::pin(future::ready(()))
    }
}

/// Thinks for 1200ms, then answers from the script.
struct SlowJudge(Shared, RecordingJudge);

impl Judge for SlowJudge {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        self.0.pass(1200);
        self.1.ask(screen, questions)
    }
}

/// Takes 300ms to touch and 500ms to observe, otherwise a [`FakeBrowser`].
struct SlowBrowser(Shared, FakeBrowser);

impl Browser for SlowBrowser {
    fn observe(&mut self) -> Pending<'_, Result<Observation, BrowserError>> {
        self.0.pass(500);
        self.1.observe()
    }

    fn touch<'a>(&'a mut self, touch: &'a Touch) -> Pending<'a, Result<(), BrowserError>> {
        self.0.pass(300);
        self.1.touch(touch)
    }

    fn screenshot(&mut self) -> Pending<'_, Result<Image, BrowserError>> {
        self.1.screenshot()
    }
}

/// The step, the operation, who asked, and the times of each act.
fn timings(record: &Record) -> Vec<(u64, &str, Cause, Duration, Duration)> {
    let acts = record.acts.iter();
    acts.map(|a| (a.step, a.op.as_str(), a.cause, a.act, a.settle))
        .collect()
}

/// Runs [`Fakes::new`] with a judge, a browser and a clock that take time.
async fn slow_run(fakes: &mut Fakes) -> (Result<ExitReason, RunError>, Record) {
    let time = Shared::default();
    let mut browser = SlowBrowser(time.clone(), mem::take(&mut fakes.browser));
    let mut judge = SlowJudge(time.clone(), mem::take(&mut fakes.judge));
    let (store, redactor) = (&mut fakes.store, &fakes.redactor);
    let clock = &mut TickingClock(time);
    run_on(&mut browser, &mut judge, clock, store, redactor, rules()).await
}

#[tokio::test]
async fn records_step_timings_with_fake_clock() {
    let mut fakes = Fakes::new();
    let (reason, record) = slow_run(&mut fakes).await;
    assert_eq!(reason, Ok(ExitReason::BudgetExceeded), "one decision");
    let calls: Vec<_> = record.calls.iter().map(|c| (c.step, c.think)).collect();
    let ms = Duration::from_millis;
    assert_eq!(calls, [(1, ms(1200))], "the judge thought");
    let timed = (1, "click", Cause::Judge, ms(300), ms(500));
    assert_eq!(
        timings(&record),
        [timed],
        "the touch and the page took time"
    );
    let asked = fakes.store.lines.first().map(|l| l["think_ms"].clone());
    assert_eq!(asked, Some(1200.into()), "the question line has the time");
}

/// Stands still, and records each time it is asked to sleep until.
#[derive(Debug, Default)]
struct SleepyClock {
    now: Duration,
    sleeps: Vec<Duration>,
}

impl Clock for SleepyClock {
    fn elapsed(&self) -> Duration {
        self.now
    }

    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        self.sleeps.push(at);
        self.now = at;
        Box::pin(future::ready(()))
    }
}

/// Scrolls down three times on the same page, with two accesses an hour
/// and a minute to run.
pub(super) fn crowded(fakes: &mut Fakes) -> Rules {
    let scroll = ("scroll_down", 0.9, "1");
    fakes.judge.script = script(&[scroll, scroll, scroll]);
    fakes.stay_on(&page("/"));
    let mut rules = limited(&[Op::ScrollDown], 20, 2);
    rules.max_duration = Some(Duration::from_secs(60));
    rules
}

#[tokio::test]
async fn a_window_opening_after_the_deadline_ends_without_sleeping() {
    let mut fakes = Fakes::new();
    let rules = crowded(&mut fakes);
    let mut clock = SleepyClock::default();
    let (browser, judge) = (&mut fakes.browser, &mut fakes.judge);
    let (store, redactor) = (&mut fakes.store, &fakes.redactor);
    let (reason, _) = run_on(browser, judge, &mut clock, store, redactor, rules).await;
    assert_eq!(reason, Ok(ExitReason::TimeExceeded), "out of time");
    assert_eq!(
        clock.sleeps,
        Vec::<Duration>::new(),
        "no window past the deadline"
    );
    assert_eq!(fakes.browser.touches.len(), 2, "two accesses in the window");
    assert_eq!(fakes.picked(&["waited"]), Vec::new(), "no wait is logged");
}
