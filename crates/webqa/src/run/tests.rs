//! Tests of the run loop with fake ports.

use std::collections::VecDeque;
use std::num::NonZeroU32;
use std::time::Duration;
use std::{future, iter};

use url::Url;
use webqa_core::config::raw::Op;
use webqa_core::guard::domain::AllowedOrigin;
use webqa_core::judge::QuestionId;
use webqa_core::machine::ExitReason;
use webqa_core::ports::{Observation, Pending};
use webqa_core::redact::Rule;
use webqa_core::screen::ElementId;
use webqa_judge::scripted::Scripted;

mod events;
mod interrupt;
mod issues;
mod material;
mod perform;
mod probes;
mod shots;
mod steps;
mod timing;
mod writes;

use super::fakes::{self, FakeBrowser, FakeClock, MemoryStore, RecordingJudge};
use super::*;

/// The user who never interrupts.
struct Never;

impl Interrupt for Never {
    fn wait(&mut self) -> Pending<'_, ()> {
        Box::pin(future::pending())
    }
}

struct Fakes {
    browser: FakeBrowser,
    judge: RecordingJudge,
    clock: FakeClock,
    store: MemoryStore,
    redactor: Redactor,
    rules: Rules,
    detect: Detect,
    /// What the last run recorded.
    record: Record,
}

impl Fakes {
    /// The page shows `保存` and a link with a mail address, stays while
    /// `保存` is looked up again, and goes to `/done` after the touch; the
    /// judge chooses to click `保存`, with one decision.
    fn new() -> Self {
        let rule = Rule::literal("メールアドレス", "taro@example.test");
        Self {
            browser: FakeBrowser {
                pages: VecDeque::from([page("/"), page("/"), page("/done")]),
                ..FakeBrowser::default()
            },
            judge: RecordingJudge {
                script: script(&[("click", 0.9, "1")]),
                ..RecordingJudge::default()
            },
            clock: FakeClock {
                now: Duration::from_millis(1500),
            },
            store: MemoryStore::default(),
            redactor: Redactor::new(&[rule]).unwrap(),
            rules: rules(),
            detect: Detect::default(),
            record: Record::default(),
        }
    }

    fn ports(&mut self) -> Ports<'_> {
        Ports::new(
            &mut self.browser,
            &mut self.judge,
            &mut self.clock,
            &mut self.store,
            &self.redactor,
        )
        .detecting(self.detect.clone())
    }

    /// Runs under `self.rules` until the core finishes, and keeps the record.
    async fn explore(&mut self) -> Result<Outcome, RunError> {
        let rules = self.rules.clone();
        let mut ports = self.ports();
        let outcome = explore(&mut ports, &mut Never, rules, Memory::default()).await;
        let record = ports.record().clone();
        self.record = record;
        outcome
    }

    async fn reason(&mut self) -> Result<ExitReason, RunError> {
        self.explore().await.map(|outcome| outcome.reason)
    }

    /// Shows `page` every time it is observed.
    fn stay_on(&mut self, page: &Observation) {
        self.browser.pages = iter::repeat_n(page.clone(), 10).collect();
    }

    fn lines(&self) -> Vec<Line> {
        let lines = self.store.lines.iter().cloned();
        lines.map(|v| serde_json::from_value(v).unwrap()).collect()
    }

    /// The kind of each line, in order.
    fn kinds(&self) -> Vec<String> {
        let kinds = self.store.lines.iter().map(|v| v["kind"].clone());
        kinds.map(|k| k.as_str().unwrap().to_owned()).collect()
    }

    /// The step and the event of each line whose kind is one of `kinds`.
    fn picked(&self, kinds: &[&str]) -> Vec<(Option<u64>, Event)> {
        let lines = self.lines().into_iter().zip(self.kinds());
        let wanted = lines.filter(|(_, kind)| kinds.contains(&kind.as_str()));
        wanted.map(|(line, _)| (line.step, line.event)).collect()
    }
}

/// Runs `rules` on the ports given, from the start of the run.
async fn run_on(
    browser: &mut dyn Browser,
    judge: &mut dyn Judge,
    clock: &mut dyn Clock,
    store: &mut MemoryStore,
    redactor: &Redactor,
    rules: Rules,
) -> (Result<ExitReason, RunError>, Record) {
    let mut ports = Ports::new(browser, judge, clock, store, redactor);
    let outcome = explore(&mut ports, &mut Never, rules, Memory::default()).await;
    (outcome.map(|o| o.reason), ports.record().clone())
}

/// Answers each step with the operation `op` at `p` on element `target`,
/// and no error shown.
fn script(steps: &[(&str, f64, &str)]) -> Scripted {
    steps
        .iter()
        .fold(Scripted::default(), |script, &(op, p, target)| {
            script
                .then(QuestionId::operation(), [(op, p)])
                .then(QuestionId::target(), [(target, 0.8)])
                .then(QuestionId::error_shown(), [("true", 0.0)])
        })
}

/// Clicking and going back, with one decision.
fn rules() -> Rules {
    limited(&[Op::Click, Op::Back], 1, 10)
}

/// The operation kinds `ops`, with `decisions` and `requests` an hour.
fn limited(ops: &[Op], decisions: u32, requests: u32) -> Rules {
    let limits = [3, 3, decisions, requests].map(|n| NonZeroU32::new(n).unwrap());
    let app = AllowedOrigin::parse("https://app.example.test").unwrap();
    fakes::rules(ops, limits, vec![app])
}

fn url(path: &str) -> Url {
    Url::parse("https://app.example.test")
        .unwrap()
        .join(path)
        .unwrap()
}

fn page(path: &str) -> Observation {
    fakes::page(
        &url(path),
        &[("button", "保存"), ("link", "taro@example.test に送る")],
    )
}

/// A page at `/` with the one button `name`, the console holding `console`.
fn one(name: &str, console: &[&str]) -> Observation {
    let mut page = fakes::page(&url("/"), &[("button", name)]);
    page.console = console.iter().map(|&e| e.to_owned()).collect();
    page
}

fn save() -> ElementId {
    ElementId {
        role: "button".to_owned(),
        name: "保存".to_owned(),
        nth: 0,
    }
}
