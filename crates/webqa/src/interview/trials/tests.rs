//! Tests of the loop outside the trials, with fake ports.

use std::collections::VecDeque;
use std::env::{temp_dir, var_os};
use std::ffi::OsString;
use std::fs;
use std::future;
use std::iter;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::PathBuf;
use std::process;
use std::time::Duration;

use url::Url;
use webqa_core::config::interview::{Interview, Reset};
use webqa_core::config::persona::Persona;
use webqa_core::config::purpose::Purpose;
use webqa_core::config::raw::Op;
use webqa_core::guard::domain::AllowedOrigin;
use webqa_core::interview::ledger::{Goal, Head, Ledger, read};
use webqa_core::judge::QuestionId;
use webqa_core::ports::{Observation, Pending};
use webqa_core::redact::Redactor;
use webqa_judge::scripted::Scripted;

mod ends;
mod opening;
mod resets;
mod stubs;
mod trials;
mod waits;
mod windows;

#[path = "../../../tests/support/launch.rs"]
mod launch;

use self::launch::FakeLaunch;
use self::stubs::{Fires, Never, Stalled};
use super::*;
use crate::run::fakes::{self, FakeBrowser, FakeClock, MemoryStore, RecordingJudge};

const ORIGIN: &str = "https://app.example.test";

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reset/");

/// A fresh directory for the test `name`.
fn scratch(name: &str) -> PathBuf {
    let dir = temp_dir().join(format!("webqa-trials-{}", process::id()));
    let dir = dir.join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// A variable of the reset command's environment.
fn var(name: &str, value: impl Into<OsString>) -> (OsString, OsString) {
    (OsString::from(name), value.into())
}

/// The page every trial starts on.
fn top() -> Observation {
    let url = Url::parse("https://app.example.test/").unwrap();
    fakes::page(&url, &[("button", "保存"), ("link", "ヘルプ")])
}

fn nz(n: u32) -> NonZeroU32 {
    NonZeroU32::new(n).unwrap()
}

fn persona() -> Persona {
    Persona {
        name: "田中".to_owned(),
        profile: "事務職".to_owned(),
        it_literacy: "苦手".to_owned(),
        wants: "経費を申請する".to_owned(),
        avoids: vec!["設定を触らない".to_owned()],
    }
}

/// The settings of `trials` trials of an inferred purpose, with 100 accesses
/// an hour and no reset command.
fn settings(trials: u32) -> Interview {
    Interview {
        persona: persona(),
        purpose: Purpose::inferred(),
        trials: nz(trials),
        hesitation_threshold: 0.5,
        confidence_threshold: 0.5,
        stuck_threshold: nz(5),
        requests_per_hour: nz(100),
        max_natural_questions: nz(10),
        reset: None,
    }
}

fn plan(trials: u32) -> Plan {
    let limits = [5, 5, 20, 100].map(nz);
    let app = AllowedOrigin::parse(ORIGIN).unwrap();
    Plan {
        interview: settings(trials),
        rules: fakes::rules(&[Op::Click], limits, vec![app]),
        start: Redactor::default().text("https://app.example.test/"),
        viewport: fakes::VIEWPORT,
        previous: None,
        max_duration: None,
        env: Vec::new(),
        withheld: Vec::new(),
    }
}

/// The judge chooses the operations `ops` in turn, on the first element.
fn script(ops: &[&str]) -> Scripted {
    ops.iter().fold(Scripted::default(), |script, &op| {
        script
            .then(QuestionId::operation(), [(op, 0.9)])
            .then(QuestionId::target(), [("1", 0.8)])
            .then(QuestionId::error_shown(), [("true", 0.0)])
            .then(QuestionId::goal_reached(), [("true", 0.0)])
    })
}

/// The judge says the purpose is reached after the first look, `times` times.
fn done(times: usize) -> Scripted {
    script(&vec!["done"; times])
}

/// A browser that shows the top page forever.
fn browser() -> FakeBrowser {
    FakeBrowser {
        pages: iter::repeat_n(top(), 30).collect::<VecDeque<_>>(),
        ..FakeBrowser::default()
    }
}

/// The fake ports and the plan of one run.
struct World {
    launch: FakeLaunch,
    judge: RecordingJudge,
    clock: FakeClock,
    store: MemoryStore,
    redactor: Redactor,
    plan: Plan,
}

impl World {
    /// `trials` trials, each ending when the judge says it is done.
    fn new(trials: u32) -> Self {
        let count = usize::try_from(trials).unwrap();
        Self {
            launch: FakeLaunch {
                browsers: iter::repeat_with(browser).take(count).collect(),
                ..FakeLaunch::default()
            },
            judge: RecordingJudge {
                script: done(count),
                ..RecordingJudge::default()
            },
            clock: FakeClock::default(),
            store: MemoryStore::default(),
            redactor: Redactor::default(),
            plan: plan(trials),
        }
    }

    async fn run(&mut self) -> Ending {
        self.run_with(&mut Never).await
    }

    async fn run_with(&mut self, interrupt: &mut dyn Interrupt) -> Ending {
        interview(
            &mut self.launch,
            &mut self.judge,
            &mut self.clock,
            &mut self.store,
            &self.redactor,
            interrupt,
            &self.plan,
        )
        .await
    }

    /// The ledger written, read back.
    fn ledger(&self) -> Option<Ledger> {
        let text = self.store.ledger.as_deref()?;
        Some(read(text).unwrap())
    }

    /// A world of `trials` trials that run the fixture `script` before each
    /// one, with a timeout of `timeout` seconds and `extra` as its
    /// environment besides its output file. The output file is in a fresh
    /// directory for `name`.
    fn resetting(trials: u32, name: &str, script: &str, extra: &[(OsString, OsString)]) -> Self {
        let mut world = Self::new(trials);
        let out = scratch(name).join("out");
        world.plan.interview.reset = Some(Reset {
            program: format!("{FIXTURES}{script}"),
            args: Vec::new(),
            timeout_secs: NonZeroU64::new(5).unwrap(),
        });
        let path = var_os("PATH").unwrap_or_default();
        let base = [var("RESET_OUT", &out), var("PATH", path)];
        world.plan.env = base.into_iter().chain(extra.iter().cloned()).collect();
        world
    }

    /// The file the reset command writes to.
    fn out(&self) -> PathBuf {
        let (_, out) = &self.plan.env[0];
        PathBuf::from(out)
    }

    /// A world of `trials` trials whose window holds two accesses: the first
    /// page of a trial and one more, so the second trial has to wait.
    fn tight(trials: u32) -> Self {
        let mut world = Self::new(trials);
        world.plan.interview.requests_per_hour = nz(2);
        world.plan.rules.requests_per_hour = nz(2);
        world
    }

    /// The kind of each line of the log that is one of `kinds`, in order.
    fn picked(&self, kinds: &[&str]) -> Vec<String> {
        let all = self.kinds();
        all.into_iter()
            .filter(|kind| kinds.contains(&kind.as_str()))
            .collect()
    }

    /// The kind of each line of the log, in order.
    fn kinds(&self) -> Vec<String> {
        let kinds = self.store.lines.iter().map(|line| line["kind"].clone());
        kinds
            .map(|kind| kind.as_str().unwrap().to_owned())
            .collect()
    }
}

/// The head of a ledger of this persona and an inferred purpose after
/// `finished` finished trials.
fn head(finished: usize) -> Head {
    Head {
        persona: "田中".to_owned(),
        goal: Goal::Inferred(Vec::new()),
        start: "https://app.example.test/".to_owned(),
        finished,
        images: false,
    }
}

const fn minutes(n: u64) -> Duration {
    Duration::from_secs(n * 60)
}
