//! Runs a scene once, with fakes tied to one gate.

use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use tokio::sync::Notify;
use webqa::exit;
use webqa::run::session::{Ending, Plan, Start, conduct};
use webqa::run::{Detect, Ports};
use webqa::setup;
use webqa::store::report::{Meta, Price};
use webqa_core::issue::probes;
use webqa_core::machine::Rules;
use webqa_core::redact::Redactor;

use super::Ran;
use crate::browser::SceneBrowser;
use crate::clock::{SceneClock, millis};
use crate::fakes::MemoryStore;
use crate::interruption::SceneInterrupt;
use crate::judge::SceneJudge;
use crate::page::PageSpec;
use crate::scene::{Scene, Stall, found};
use crate::store::SceneStore;

/// What the run was told: the rules, the redaction and the confidence the
/// error display needs.
struct Setting {
    rules: Rules,
    redactor: Redactor,
    error_confidence: f64,
}

/// The file the configuration of a scene reads.
const AUTH: &str = "./auth.json";

/// The fakes of a run, tied to one gate: whichever waits forever wakes the
/// interrupt once.
pub struct Fakes {
    judge: SceneJudge,
    browser: SceneBrowser,
    clock: SceneClock,
    store: SceneStore,
    interrupt: SceneInterrupt,
}

impl Fakes {
    pub fn of(scene: &Scene) -> Self {
        let gate = Arc::new(Notify::new());
        let now = Arc::new(AtomicU64::new(millis(scene.clock_start)));
        let browser = browser_of(scene, &gate);
        Self {
            judge: SceneJudge::new(
                scene.replies.clone(),
                Arc::clone(&gate),
                Arc::clone(&now),
                scene.think,
            ),
            browser,
            clock: SceneClock {
                now,
                stall_sleep: scene.stall == Stall::Sleep,
                sleeps: Vec::new(),
                gate: Arc::clone(&gate),
            },
            store: SceneStore {
                inner: MemoryStore {
                    limit: scene.store_limit,
                    memories: scene.memories.clone(),
                    ..MemoryStore::default()
                },
                hold: scene.store_hold,
                held: false,
                gate: Arc::clone(&gate),
            },
            interrupt: SceneInterrupt {
                fire_on: scene.fire_on.clone(),
                gate,
                ..SceneInterrupt::default()
            },
        }
    }

    /// Runs the scene once with `setting`.
    async fn conduct(&mut self, scene: &Scene, setting: Setting) -> Ending {
        let origin = scene.origin();
        let start = scene.start.build(&origin).screen.url;
        let redactor = &setting.redactor;
        let probes = if scene.probes { probes() } else { Vec::new() };
        let mut ports = Ports::new(
            &mut self.browser,
            &mut self.judge,
            &mut self.clock,
            &mut self.store,
            redactor,
        )
        .detecting(Detect {
            error_confidence: setting.error_confidence,
            probes,
        });
        let plan = Plan {
            origin,
            meta: Meta {
                mode: scene.mode,
                started_at: Some("2026-10-18T02:00:00Z".to_owned()),
                target: redactor.url(&start),
                inputs: Vec::new(),
                price: Some(Price {
                    input_per_1k: 0.001,
                    output_per_1k: 0.002,
                }),
            },
            start: Start::Stored,
        };
        conduct(&mut ports, &mut self.interrupt, setting.rules, &plan).await
    }

    pub fn into_ran(self, ending: Option<Ending>, config_error: Option<String>, given: u64) -> Ran {
        Ran {
            code: ending.as_ref().map_or(exit::CONFIG_ERROR, exit::code),
            ending,
            config_error,
            judge: self.judge,
            browser: self.browser,
            clock: self.clock,
            store: self.store.inner,
            interrupt: self.interrupt,
            given_steps: given,
        }
    }
}

/// The browser of `scene`, tied to `gate`.
fn browser_of(scene: &Scene, gate: &Arc<Notify>) -> SceneBrowser {
    let origin = scene.origin();
    let build = |page: &PageSpec| page.build(&origin);
    let mut browser = SceneBrowser::new(
        build(&scene.start),
        scene.after.iter().map(build).collect(),
        scene.looks.iter().map(build).collect(),
        Arc::clone(gate),
    );
    browser.stall_touch = scene.stall == Stall::Touch;
    browser.fail_touch = scene.fail_touch;
    browser
}

/// The rules, the redaction and the error confidence of the scene: from its
/// YAML when it has one, else from what the Given steps set up.
fn setting(scene: &Scene) -> Result<Setting, setup::SetupError> {
    scene
        .yaml
        .as_deref()
        .map_or_else(|| Ok(plain(scene)), |yaml| configured(scene, yaml))
}

/// What the Given steps set up, with no YAML.
fn plain(scene: &Scene) -> Setting {
    Setting {
        rules: scene.rules(),
        redactor: found(
            Redactor::new(&scene.redact).ok(),
            "the redaction rules read",
        ),
        error_confidence: scene.error_confidence,
    }
}

/// What `yaml` says, checked with the environment of the scene.
fn configured(scene: &Scene, yaml: &str) -> Result<Setting, setup::SetupError> {
    let env = |name: &str| scene.env.get(name).cloned();
    let setup = setup::load(yaml, scene.mode, &env, &read_auth)?;
    Ok(Setting {
        rules: setup.rules,
        redactor: setup.redactor,
        error_confidence: setup.error_confidence,
    })
}

/// The files the configuration of a scene can read: only [`AUTH`].
fn read_auth(path: &Path) -> io::Result<String> {
    let found = (path == Path::new(AUTH)).then(|| r#"{"cookies":[],"origins":[]}"#.to_owned());
    found.ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
}

/// Runs `scene` once, with fakes.
pub async fn play(scene: &Scene) -> Ran {
    let mut fakes = Fakes::of(scene);
    let (ending, config_error) = match setting(scene) {
        Ok(setting) => (Some(fakes.conduct(scene, setting).await), None),
        Err(error) => (None, Some(error.to_string())),
    };
    fakes.into_ran(ending, config_error, scene.given_steps())
}
