//! Fake browser, judge, clock and store for running the loop without the outside.
//!
//! The acceptance tests use them, and so do the unit tests of `run`.

use std::collections::VecDeque;
use std::future;
use std::num::NonZeroU32;
use std::time::Duration;

use serde_json::Value;
use url::Url;
use webqa_core::config::raw::Op;
use webqa_core::effect::Touch;
use webqa_core::event::Mode;
use webqa_core::guard::domain::AllowedOrigin;
use webqa_core::guard::forbidden::Forbidden;
use webqa_core::guard::paywall::Paywall;
use webqa_core::judge::{Answers, JudgeError, Question};
use webqa_core::machine::Rules;
use webqa_core::ports::{
    Browser, BrowserError, Clock, Judge, Observation, Pending, Store, StoreError,
};
use webqa_core::redact::Redacted;
use webqa_core::screen::{Element, Rect, Screen, ScreenText, Viewport};
use webqa_judge::scripted::Scripted;

/// The viewport of every fake page.
pub const VIEWPORT: Viewport = Viewport {
    width: 100,
    height: 100,
};

/// The thresholds and caps of a run: freeze, stuck, decisions and accesses
/// in a window.
pub type Limits = [NonZeroU32; 4];

/// Rules of a scenario with the operation kinds `ops`, the `limits` and the
/// `allowed` origins. No time limit, the default guardrails, `back` to
/// leave a paywall.
pub fn rules(ops: &[Op], limits: Limits, allowed: Vec<AllowedOrigin>) -> Rules {
    let [freeze, stuck, decisions, requests] = limits;
    Rules {
        mode: Mode::Scenario,
        ops: ops.to_vec(),
        freeze_threshold: freeze,
        stuck_threshold: stuck,
        max_decisions: decisions,
        requests_per_hour: requests,
        max_duration: None,
        back_confidence: 0.7,
        allowed,
        forbidden: Forbidden::default(),
        paywall: Paywall::default(),
        leave_paywall: "back".to_owned(),
    }
}

/// A page at `url` with the (role, name) elements stacked from the top of the
/// viewport, 20px apart.
pub fn page(url: &Url, elements: &[(&str, &str)]) -> Observation {
    let elements = elements
        .iter()
        .zip((0..).step_by(20))
        .map(|(&(role, name), y)| element(role, name, y))
        .collect();
    Observation {
        screen: Screen {
            url: url.clone(),
            elements,
        },
        viewport: VIEWPORT,
    }
}

fn element(role: &str, name: &str, y: i64) -> Element {
    Element {
        role: role.to_owned(),
        name: name.to_owned(),
        value: None,
        rect: Some(Rect {
            x: 0,
            y,
            width: 10,
            height: 10,
        }),
    }
}

/// Records the screen texts it was sent, then answers from the script.
#[derive(Debug, Default)]
pub struct RecordingJudge {
    /// The screen texts sent, in order.
    pub texts: Vec<String>,
    /// Where the answers come from.
    pub script: Scripted,
}

impl Judge for RecordingJudge {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        self.texts.push(screen.as_inner().as_str().to_owned());
        self.script.ask(screen, questions)
    }
}

/// Shows the pages in order and records the touches. Fails when no page is
/// left, and fails every touch when broken.
#[derive(Debug, Default)]
pub struct FakeBrowser {
    /// The pages still to show.
    pub pages: VecDeque<Observation>,
    /// The touches received, in order.
    pub touches: Vec<Touch>,
    /// Whether touches fail.
    pub broken: bool,
}

impl Browser for FakeBrowser {
    fn observe(&mut self) -> Pending<'_, Result<Observation, BrowserError>> {
        Box::pin(future::ready(self.pages.pop_front().ok_or(BrowserError)))
    }

    fn touch<'a>(&'a mut self, touch: &'a Touch) -> Pending<'a, Result<(), BrowserError>> {
        self.touches.push(touch.clone());
        let result = if self.broken {
            Err(BrowserError)
        } else {
            Ok(())
        };
        Box::pin(future::ready(result))
    }
}

/// Stands still until told to sleep.
#[derive(Debug, Default)]
pub struct FakeClock {
    /// Time since the run started.
    pub now: Duration,
}

impl Clock for FakeClock {
    fn elapsed(&self) -> Duration {
        self.now
    }

    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        self.now = at;
        Box::pin(future::ready(()))
    }
}

/// Keeps the lines in memory. Fails once it holds `limit` lines.
#[derive(Debug, Default)]
pub struct MemoryStore {
    /// The lines written, including the ones that failed.
    pub lines: Vec<Value>,
    /// How many lines it takes; `None` takes them all.
    pub limit: Option<usize>,
}

impl Store for MemoryStore {
    fn write_event<'a>(
        &'a mut self,
        line: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        let full = self.limit.is_some_and(|limit| self.lines.len() >= limit);
        self.lines.push(line.as_inner().clone());
        let result = if full { Err(StoreError) } else { Ok(()) };
        Box::pin(future::ready(result))
    }
}
