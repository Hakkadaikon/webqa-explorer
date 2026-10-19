//! Tests of the run loop with fake ports.

use std::collections::VecDeque;
use std::num::NonZeroU32;

use url::Url;
use webqa_core::config::raw::Op;
use webqa_core::guard::domain::AllowedOrigin;
use webqa_core::judge::{JudgeError, QuestionId};
use webqa_core::machine::{Anomaly, ExitReason, Note};
use webqa_core::ports::Observation;
use webqa_core::redact::Rule;
use webqa_core::screen::ElementId;
use webqa_judge::scripted::Scripted;

mod perform;

use super::fakes::{self, FakeBrowser, FakeClock, MemoryStore, RecordingJudge};
use super::*;

struct Fakes {
    browser: FakeBrowser,
    judge: RecordingJudge,
    clock: FakeClock,
    store: MemoryStore,
    redactor: Redactor,
}

impl Fakes {
    /// The page shows `保存` and a link with a mail address, stays while
    /// `保存` is looked up again, and goes to `/done` after the touch; the
    /// judge chooses to click `保存`.
    fn new() -> Self {
        let rule = Rule::literal("メールアドレス", "taro@example.test");
        let script = Scripted::default()
            .then(QuestionId::operation(), [("click", 0.9), ("back", 0.1)])
            .then(QuestionId::target(), [("1", 0.8), ("2", 0.2)])
            .then(QuestionId::error_shown(), [("true", 0.0)]);
        Self {
            browser: FakeBrowser {
                pages: VecDeque::from([page("/"), page("/"), page("/done")]),
                ..FakeBrowser::default()
            },
            judge: RecordingJudge {
                script,
                ..RecordingJudge::default()
            },
            clock: FakeClock {
                now: Duration::from_millis(1500),
            },
            store: MemoryStore::default(),
            redactor: Redactor::new(&[rule]).unwrap(),
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
    }

    /// Runs with one decision, so the run ends after the first operation.
    async fn explore(&mut self) -> Result<Outcome, RunError> {
        explore(&mut self.ports(), rules(), Memory::default()).await
    }

    async fn reason(&mut self) -> Result<ExitReason, RunError> {
        self.explore().await.map(|outcome| outcome.reason)
    }

    fn lines(&self) -> Vec<Line> {
        let lines = self.store.lines.iter().cloned();
        lines.map(|v| serde_json::from_value(v).unwrap()).collect()
    }
}

/// Clicking and going back, with one decision.
fn rules() -> Rules {
    let limits = [3, 3, 1, 10].map(|n| NonZeroU32::new(n).unwrap());
    let app = AllowedOrigin::parse("https://app.example.test").unwrap();
    fakes::rules(&[Op::Click, Op::Back], limits, vec![app])
}

fn page(path: &str) -> Observation {
    let url = Url::parse("https://app.example.test")
        .unwrap()
        .join(path)
        .unwrap();
    fakes::page(
        &url,
        &[("button", "保存"), ("link", "taro@example.test に送る")],
    )
}

fn line(seq: u64, event: Event) -> Line {
    Line {
        seq,
        elapsed_ms: 1500,
        step: Some(1),
        event,
    }
}

#[tokio::test]
async fn one_step_asks_once_touches_once_and_logs_the_decision_and_the_result() {
    let mut fakes = Fakes::new();
    let ended = Ok(ExitReason::BudgetExceeded);
    assert_eq!(fakes.reason().await, ended, "no decision left");
    assert_eq!(
        fakes.judge.texts,
        vec!["1: button \"保存\"\n2: link \"[[メールアドレス]] に送る\"".to_owned()],
        "asked once, with the address masked"
    );
    let save = ElementId {
        role: "button".to_owned(),
        name: "保存".to_owned(),
        nth: 0,
    };
    let touch = Touch {
        op: "click".to_owned(),
        target: Some(save),
        confidence: 0.9,
    };
    assert_eq!(fakes.browser.touches, vec![touch], "touched once");
    let decided = Event::Decided {
        op: "click".to_owned(),
        target: Some("保存".to_owned()),
        p: 0.9,
    };
    let executed = Event::Executed {
        changed: true,
        accesses: 1,
    };
    assert_eq!(
        fakes.lines(),
        vec![line(0, decided), line(1, executed)],
        "one line for the decision, one for the result"
    );
}

#[tokio::test]
async fn touching_the_same_page_logs_no_change() {
    let mut fakes = Fakes::new();
    fakes.browser.pages = VecDeque::from([page("/"), page("/"), page("/")]);
    let ended = Ok(ExitReason::BudgetExceeded);
    assert_eq!(fakes.reason().await, ended, "no decision left");
    let unchanged = line(
        1,
        Event::Executed {
            changed: false,
            accesses: 1,
        },
    );
    assert_eq!(fakes.lines().get(1), Some(&unchanged), "the page stayed");
}

#[tokio::test]
async fn judge_failure_ends_the_run_without_touching() {
    let mut fakes = Fakes::new();
    fakes.judge.script = Scripted::default();
    let outcome = fakes.explore().await.unwrap();
    let ended = ExitReason::BudgetExceeded;
    assert_eq!(outcome.reason, ended, "no decision left");
    let failed = [Note::Anomaly(Anomaly::Judge(JudgeError::Exhausted))];
    assert_eq!(outcome.state.notes(), failed, "the core records the error");
    assert_eq!(fakes.browser.touches, Vec::new(), "nothing touched");
}

async fn fails_with(breaks: fn(&mut Fakes), error: RunError) {
    let mut fakes = Fakes::new();
    breaks(&mut fakes);
    assert_eq!(fakes.reason().await, Err(error), "the run stops");
}

#[tokio::test]
async fn no_page_to_start_from_stops_the_run() {
    fails_with(|f| f.browser.pages.clear(), BrowserError.into()).await;
}

#[tokio::test]
async fn a_failed_touch_stops_the_run() {
    fails_with(|f| f.browser.broken = true, BrowserError.into()).await;
}

#[tokio::test]
async fn no_page_after_the_touch_stops_the_run() {
    fails_with(|f| drop(f.browser.pages.pop_back()), BrowserError.into()).await;
}

#[tokio::test]
async fn an_unwritable_decision_stops_the_run() {
    fails_with(|f| f.store.limit = Some(0), StoreError.into()).await;
}

#[tokio::test]
async fn an_unwritable_result_stops_the_run() {
    fails_with(|f| f.store.limit = Some(1), StoreError.into()).await;
}
