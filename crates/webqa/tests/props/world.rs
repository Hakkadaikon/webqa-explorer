//! The ports and the trace of a run of a case.

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};
use serde_json::Value;
use tokio::runtime::Builder;
use url::Url;
use webqa::interview::trials::{Ending, Plan, interview};
use webqa_core::interview::ledger::read;
use webqa_core::redact::Redactor;

use super::case::{Case, cases};
use super::clocks::{Fires, Opening, Sleep, Thinking, Time, Virtual};
use super::plan::{Fallible, plan, script};
use crate::fakes::{self, MemoryStore, RecordingJudge};

pub const SCRATCH: &str = concat!(env!("CARGO_TARGET_TMPDIR"), "/trials_props");
const SEED: [u8; 32] = [7; 32];
const CASES: u32 = 48;

/// The times at which the reset command started: the reading of the clock
/// just before the command wrote its file.
fn reset_starts(readings: &[(u64, usize)]) -> Vec<u64> {
    let pairs = readings.iter().zip(readings.iter().skip(1));
    let started = pairs.filter(|(before, after)| after.1 > before.1);
    started.map(|(before, _)| before.0).collect()
}

/// What a run of a case did.
pub struct Trace {
    pub ending: Ending,
    /// The lines of the event log.
    pub lines: Vec<Value>,
    /// The ledger's count of finished trials.
    pub finished: usize,
    pub sleeps: Vec<Sleep>,
    /// When each browser started opening.
    pub launches: Vec<u64>,
    /// When each reset command started.
    pub resets: Vec<u64>,
    pub interrupts: usize,
    /// How many interruptions the run saw.
    pub fired: usize,
}

/// The ports and the plan of a run.
pub struct Run {
    pub launch: Opening,
    pub judge: Thinking,
    pub clock: Virtual,
    pub store: MemoryStore,
    pub interrupt: Fires,
    pub plan: Plan,
}

/// The clock and the launch of `case`, whose reset command writes to `out`.
fn timing(case: &Case, out: PathBuf) -> Fallible<(Virtual, Opening)> {
    let url = Url::parse("https://app.example.test/")?;
    let page = fakes::page(&url, &[("button", "保存"), ("link", "ヘルプ")]);
    let now: Time = Arc::new(AtomicU64::new(0));
    let launches = Arc::new(AtomicUsize::new(0));
    let clock = Virtual {
        now: Arc::clone(&now),
        out,
        launches: Arc::clone(&launches),
        sleeps: Vec::new(),
        readings: RefCell::new(Vec::new()),
    };
    let launch = Opening {
        now,
        minutes: case.launch,
        launches,
        started: Vec::new(),
        page,
    };
    Ok((clock, launch))
}

/// The ports and the plan of `case` as the case number `n`.
fn prepare(case: &Case, n: usize) -> Fallible<Run> {
    let dir = Path::new(SCRATCH).join(format!("case-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    let out = dir.join("out");
    let (clock, launch) = timing(case, out.clone())?;
    let inner = RecordingJudge {
        script: script(case),
        ..RecordingJudge::default()
    };
    let judge = Thinking {
        inner,
        now: Arc::clone(&launch.now),
        minutes: case.think,
    };
    let interrupt = Fires {
        from: case.interrupt_from,
        calls: 0,
        fired: Arc::default(),
    };
    let plan = plan(case, &out)?;
    let store = MemoryStore::default();
    Ok(Run {
        launch,
        judge,
        clock,
        store,
        interrupt,
        plan,
    })
}

/// Runs `case` as the case number `n`.
async fn simulate(case: &Case, n: usize) -> Fallible<Trace> {
    let mut run = prepare(case, n)?;
    let ending = interview(
        &mut run.launch,
        &mut run.judge,
        &mut run.clock,
        &mut run.store,
        &Redactor::default(),
        &mut run.interrupt,
        &run.plan,
    )
    .await;
    let ledger = run.store.ledger.as_deref().map(read);
    Ok(Trace {
        ending,
        lines: run.store.lines,
        finished: ledger.and_then(Result::ok).map_or(0, |l| l.head.finished),
        resets: reset_starts(&run.clock.readings.borrow()),
        sleeps: run.clock.sleeps,
        launches: run.launch.started,
        interrupts: run.interrupt.calls,
        fired: run.interrupt.fired.load(Ordering::SeqCst),
    })
}

/// A runner of a fixed seed and a fixed number of cases.
fn runner() -> TestRunner {
    let config = Config {
        cases: CASES,
        failure_persistence: None,
        ..Config::default()
    };
    TestRunner::new_with_rng(config, TestRng::from_seed(RngAlgorithm::ChaCha, &SEED))
}

/// Runs `check` on each of the cases of a fixed seed, and fails with the
/// smallest case that breaks it.
pub fn property(check: impl Fn(&Case, &Trace) -> Result<(), TestCaseError>) -> Fallible<()> {
    static RUNS: AtomicUsize = AtomicUsize::new(0);
    let runtime = Builder::new_current_thread().enable_all().build()?;
    runner().run(&cases(), |case| {
        let n = RUNS.fetch_add(1, Ordering::SeqCst);
        check(&case, &runtime.block_on(simulate(&case, n))?)
    })?;
    Ok(())
}
