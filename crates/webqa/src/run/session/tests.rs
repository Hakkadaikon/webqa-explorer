//! Tests of one whole run with fake ports.

use std::collections::{BTreeSet, VecDeque};
use std::iter;
use std::num::NonZeroU32;
use std::sync::Arc;

use serde_json::{Value, json};
use tokio::sync::Notify;
use url::Url;
use webqa_core::config::raw::Op;
use webqa_core::event::Mode;
use webqa_core::explore::Memory;
use webqa_core::guard::domain::AllowedOrigin;
use webqa_core::judge::QuestionId;
use webqa_core::machine::{ExitReason, Rules};
use webqa_core::ports::{Judge, Observation, Store};
use webqa_core::redact::Redactor;
use webqa_core::screen::ElementId;
use webqa_judge::scripted::Scripted;

mod broken;
mod failures;
mod fakes;
mod records;
mod start;

use self::fakes::{Counting, Gate, Hangs, Windowed};
use super::super::fakes::{self as base, FakeBrowser, FakeClock, MemoryStore, RecordingJudge};
use super::*;
use crate::exit;
use crate::store::report::Meta;

const ORIGIN: &str = "https://app.example.test";

/// The page every run starts on: a save button and three links.
fn top() -> Observation {
    let url = Url::parse("https://app.example.test/").unwrap();
    base::page(
        &url,
        &[
            ("button", "保存"),
            ("link", "ヘルプ"),
            ("link", "設定"),
            ("link", "検索"),
        ],
    )
}

fn plan() -> Plan {
    Plan {
        origin: ORIGIN.to_owned(),
        meta: Meta {
            mode: Mode::Explore,
            started_at: Some("2026-10-18T02:00:00Z".to_owned()),
            target: Redactor::default().text("https://app.example.test/"),
            inputs: Vec::new(),
            price: None,
        },
        start: Start::Stored,
    }
}

/// Exploring, with `requests` accesses an hour.
fn rules(requests: u32) -> Rules {
    let limits = [5, 5, 20, requests].map(|n| NonZeroU32::new(n).unwrap());
    let app = AllowedOrigin::parse(ORIGIN).unwrap();
    Rules {
        mode: Mode::Explore,
        ..base::rules(&[Op::Click], limits, vec![app])
    }
}

/// The three steps of the script: click the first three elements, one
/// after the other. Each one shows no error. The explorer takes an untouched
/// element first, so a second click on `保存` would not be an option, and the
/// fourth element is left for a fourth question.
fn clicks() -> Scripted {
    ["1", "2", "3"]
        .iter()
        .fold(Scripted::default(), |script, &target| {
            script
                .then(QuestionId::operation(), [("click", 0.9)])
                .then(QuestionId::target(), [(target, 0.8)])
                .then(QuestionId::error_shown(), [("true", 0.0)])
        })
}

/// What the run waits on when the user interrupts it.
#[derive(Clone, Copy)]
enum Wait {
    /// The start of the next step.
    Step,
    /// The answer of the judge to the question after this many answers.
    Question(usize),
    /// The time window to open.
    Window,
}

/// The waits, and the step the run was at when it was interrupted.
const WAITS: [(Wait, u64); 3] = [(Wait::Step, 3), (Wait::Question(3), 4), (Wait::Window, 3)];

/// Shows `page` every time.
fn browser_on(page: &Observation) -> FakeBrowser {
    FakeBrowser {
        pages: iter::repeat_n(page.clone(), 30).collect::<VecDeque<_>>(),
        ..FakeBrowser::default()
    }
}

/// Follows the script, and hangs after the answers `wait` allows.
fn judge_for(wait: Wait, gate: &Arc<Notify>) -> impl Judge + use<> {
    let answers = match wait {
        Wait::Question(n) => n,
        Wait::Step | Wait::Window => usize::MAX,
    };
    Hangs {
        answers,
        gate: Arc::clone(gate),
        inner: RecordingJudge {
            script: clicks(),
            ..RecordingJudge::default()
        },
    }
}

/// Interrupts at the tenth wait when `wait` is the next step, or when the
/// gate is let through.
fn user_for(wait: Wait, gate: &Arc<Notify>) -> Box<dyn Interrupt> {
    if matches!(wait, Wait::Step) {
        Box::new(Counting {
            calls: 0,
            fire_on: BTreeSet::from([10]),
        })
    } else {
        Box::new(Gate(Arc::clone(gate)))
    }
}

/// Runs on `store`, interrupted while waiting on `wait`, or by `gate`. The
/// browser shows `page` every time, and the script clicks the first three
/// elements.
async fn interrupted(
    wait: Wait,
    page: &Observation,
    gate: &Arc<Notify>,
    store: &mut dyn Store,
) -> Ending {
    let mut user = user_for(wait, gate);
    run_with(user.as_mut(), wait, page, gate, store).await
}

/// [`interrupted`] with the interruptions of `user`.
async fn run_with(
    user: &mut dyn Interrupt,
    wait: Wait,
    page: &Observation,
    gate: &Arc<Notify>,
    store: &mut dyn Store,
) -> Ending {
    let mut clock = Windowed {
        inner: FakeClock::default(),
        gate: Arc::clone(gate),
    };
    let requests = if matches!(wait, Wait::Window) { 3 } else { 100 };
    let redactor = Redactor::default();
    let (mut browser, mut judge) = (browser_on(page), judge_for(wait, gate));
    let mut ports = Ports::new(&mut browser, &mut judge, &mut clock, store, &redactor);
    conduct(&mut ports, user, rules(requests), &plan()).await
}

/// The run on `top`, interrupted while waiting on `wait`, into `store`.
async fn on_top(wait: Wait, store: &mut dyn Store) -> Ending {
    interrupted(wait, &top(), &Arc::new(Notify::new()), store).await
}

fn save() -> ElementId {
    ElementId {
        role: "button".to_owned(),
        name: "保存".to_owned(),
        nth: 0,
    }
}

/// Whether the memory written for the app has `保存` operated on the top page.
fn saved_operated(store: &MemoryStore) -> bool {
    let text = store.memories.get(ORIGIN).unwrap();
    let mut memory = Memory::from_json(text, ORIGIN).unwrap();
    let screen = Redactor::default().screen(top().screen);
    let at = memory.visit(&screen, &BTreeSet::new());
    memory.is_operated(at, &save())
}

/// The lines of the store without their numbers and times. The step stays.
fn kinds(store: &MemoryStore) -> Vec<Value> {
    let bare = |line: &Value| {
        let mut line = line.clone();
        let fields = line.as_object_mut().unwrap();
        fields.remove("seq");
        fields.remove("elapsed_ms");
        line
    };
    store.lines.iter().map(bare).collect()
}

/// The first line and the last line of the log, without their numbers.
fn ends(store: &MemoryStore) -> (Option<Value>, Option<Value>) {
    let lines = kinds(store);
    (lines.first().cloned(), lines.last().cloned())
}

/// Why the run ended, which it did with its records written.
fn reason_of(ending: Ending) -> ExitReason {
    let Ending::Finished { outcome, .. } = ending else {
        panic!("the run did not finish: {ending:?}")
    };
    outcome.reason
}

#[tokio::test]
async fn interrupt_cancels_wait_and_writes_records() {
    for (wait, step) in WAITS {
        let mut store = MemoryStore::default();
        let ending = on_top(wait, &mut store).await;
        assert_eq!(exit::code(&ending), 0);
        assert_eq!(reason_of(ending), ExitReason::UserAborted);
        assert!(saved_operated(&store));
        let lines = kinds(&store);
        assert_eq!(
            lines.last(),
            Some(&json!({"kind": "run_finished", "reason": "user_aborted", "step": step}))
        );
        let first = json!({"kind": "run_started", "schema": 1, "mode": "explore", "started_at": "2026-10-18T02:00:00Z"});
        assert_eq!(lines.first(), Some(&first));
        let report = store.report.unwrap();
        assert_eq!(report["reason"], "user_aborted");
        assert_eq!(report["steps"].as_array().unwrap().len(), 3);
    }
}
