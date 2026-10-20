//! Runs a scene once, with fakes tied to one gate.

use std::collections::VecDeque;
use std::ffi::{OsStr, OsString};
use std::iter;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tokio::sync::Notify;
use webqa::exit;
use webqa::interview::trials::{Ending, Plan, interview};
use webqa::run::Detect;
use webqa_core::event::Mode;
use webqa_core::ports::Judge;
use webqa_core::redact::Redactor;

use super::Ran;
use crate::bridge::{self, RunDir};
use crate::fakes::{FakeBrowser, MemoryStore, VIEWPORT};
use crate::interruption::SceneInterrupt;
use crate::judge::SceneJudge;
use crate::launch::SceneLaunch;
use crate::page::PageSpec;
use crate::run_clock::{Resets, TrialClock};
use crate::scene::{PAGES, Scene, found};
use crate::start::{Started, start};

/// The variable that names the file of the clock, for a reset script.
const CLOCK_VAR: &str = "SCENE_CLOCK";
/// The variable that names the file the reset scripts count their runs in.
const OUT_VAR: &str = "RESET_OUT";

/// One browser for each trial: the start page, the pages after it, then
/// the last page again up to [`PAGES`].
fn browsers(scene: &Scene) -> VecDeque<FakeBrowser> {
    let origin = scene.origin();
    let build = |page: &PageSpec| page.build(&origin);
    let trials = usize::try_from(scene.trials).unwrap_or(usize::MAX);
    (0..trials)
        .map(|trial| {
            let after = scene.after.get(trial).map_or(&[][..], Vec::as_slice);
            let mut pages: VecDeque<_> = iter::once(&scene.start).chain(after).map(build).collect();
            let last = found(pages.back().cloned(), "a trial has a start page");
            pages.extend(iter::repeat_n(last, PAGES.saturating_sub(pages.len())));
            FakeBrowser {
                pages,
                ..FakeBrowser::default()
            }
        })
        .collect()
}

/// The plan of the loop, as the CLI makes it from the setup.
fn plan(started: Started, env: Vec<(OsString, OsString)>) -> Plan {
    let Started {
        setup,
        interview,
        previous,
    } = started;
    let withheld = setup
        .judge
        .api_key
        .iter()
        .map(|key| key.expose().to_owned());
    Plan {
        interview,
        max_duration: setup.rules.max_duration,
        rules: setup.rules,
        start: setup.target,
        viewport: VIEWPORT,
        previous,
        env,
        withheld: withheld.collect(),
        mode: Mode::Scenario,
        started_at: Some("2026-10-18T02:00:00Z".to_owned()),
        detect: Detect::default(),
    }
}

/// The clock of a run of `scene`, sharing `now` with the judge.
fn clock_of(scene: &Scene, run: &RunDir, now: Arc<AtomicU64>, gate: Arc<Notify>) -> TrialClock {
    let timeout = scene.reset.as_ref().map(|reset| reset.timeout_secs.get());
    TrialClock {
        now,
        stall_sleep: scene.stall_sleep,
        resets: Resets {
            timeout: timeout.map(Duration::from_secs),
            hangs_from: scene.reset_hangs_from,
            out: Some(reset_out(scene, run)),
            deadlines: 0,
        },
        file: Some(run.0.join("clock")),
        gate,
        ..TrialClock::default()
    }
}

/// The fakes of a run, tied to one gate: whichever waits forever wakes the
/// interrupt once.
struct Fakes {
    judge: SceneJudge,
    bridge: Option<Box<dyn Judge>>,
    clock: TrialClock,
    launch: SceneLaunch,
    interrupt: SceneInterrupt,
    store: MemoryStore,
    gate: Arc<Notify>,
}

impl Fakes {
    fn of(scene: &Scene, run: &RunDir) -> Self {
        let gate = Arc::new(Notify::new());
        let now = Arc::new(AtomicU64::new(0));
        let replies = scene.replies.clone();
        let mut judge = SceneJudge::new(replies, Arc::clone(&gate), Arc::clone(&now), scene.think);
        judge.images = scene.images;
        judge.thinks.clone_from(&scene.thinks);
        Self {
            judge,
            bridge: None,
            clock: clock_of(scene, run, now, Arc::clone(&gate)),
            launch: SceneLaunch {
                browsers: browsers(scene),
                hangs_on: scene.hang_launch,
                reset_out: Some(reset_out(scene, run)),
                ..SceneLaunch::default()
            },
            interrupt: SceneInterrupt {
                fire_on: scene.fire_on.clone(),
                gate: Arc::clone(&gate),
                ..SceneInterrupt::default()
            },
            store: MemoryStore {
                memories: scene.memories.clone(),
                ..MemoryStore::default()
            },
            gate,
        }
    }

    /// Runs the loop with `plan`, through the bridge if one was started.
    async fn conduct(&mut self, plan: &Plan, redactor: &Redactor) -> Ending {
        let judge: &mut dyn Judge = match self.bridge.as_mut() {
            Some(bridge) => bridge.as_mut(),
            None => &mut self.judge,
        };
        interview(
            &mut self.launch,
            judge,
            &mut self.clock,
            &mut self.store,
            redactor,
            &mut self.interrupt,
            plan,
        )
        .await
    }

    fn into_ran(self, ending: Option<Ending>, config_error: Option<String>, run: &RunDir) -> Ran {
        Ran {
            code: ending
                .as_ref()
                .map_or(exit::CONFIG_ERROR, exit::interview_code),
            ending,
            config_error,
            judge: self.judge,
            bridge: bridge::asked(&run.0),
            touches: taken(&self.launch.touches),
            openings: taken(&self.launch.openings),
            launched: self.launch.launched,
            store: self.store,
            clock: self.clock,
        }
    }
}

/// What the browsers handed in.
fn taken<T: Clone>(shared: &Mutex<Vec<T>>) -> Vec<T> {
    let all = shared.lock().unwrap_or_else(PoisonError::into_inner);
    all.clone()
}

/// The file the reset scripts count their runs in: the scene's
/// `RESET_OUT`, or else one in the run's directory.
fn reset_out(scene: &Scene, run: &RunDir) -> PathBuf {
    let given = scene
        .env
        .iter()
        .find(|(name, _)| name.as_os_str() == OsStr::new(OUT_VAR));
    given.map_or_else(|| run.0.join("reset"), |(_, out)| PathBuf::from(out))
}

/// The environment of the run: the scene's, the file of the clock, and
/// the file the reset scripts count their runs in.
fn environment(scene: &Scene, run: &RunDir) -> Vec<(OsString, OsString)> {
    let added = [
        (CLOCK_VAR, run.0.join("clock")),
        (OUT_VAR, reset_out(scene, run)),
    ];
    let added = added.map(|(name, path)| (OsString::from(name), path.into_os_string()));
    scene.env.iter().cloned().chain(added).collect()
}

impl Fakes {
    /// Ties the fakes to what the stages of starting made: the storage
    /// state, and the bridge if the scene has one.
    async fn begin(&mut self, scene: &Scene, started: &Started, run: &RunDir) {
        let setup = &started.setup;
        self.launch
            .storage_state
            .clone_from(&setup.browser.storage_state);
        if scene.bridge.is_some() {
            bridge::write_answers(&run.0, &scene.replies);
            let gate = Arc::clone(&self.gate);
            self.bridge = Some(bridge::start(&setup.judge, scene.replies.len(), gate).await);
        }
    }
}

/// Runs `scene` once, with fakes.
pub async fn play(scene: &Scene) -> Ran {
    let run = RunDir::new();
    let mut fakes = Fakes::of(scene, &run);
    let redactor = found(
        Redactor::new(&scene.redact).ok(),
        "the redaction rules read",
    );
    let env = environment(scene, &run);
    match start(scene, &env, &run.0) {
        Ok(started) => {
            fakes.begin(scene, &started, &run).await;
            let ended = fakes.conduct(&plan(started, env), &redactor).await;
            fakes.into_ran(Some(ended), None, &run)
        }
        Err(error) => fakes.into_ran(None, Some(error), &run),
    }
}
