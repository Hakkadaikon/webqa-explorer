//! Runs a scene once, with fakes tied to one gate.

use std::collections::VecDeque;
use std::iter;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, PoisonError};

use tokio::sync::Notify;
use webqa::exit;
use webqa::interview::trials::{Ending, Plan, interview};
use webqa::run::Detect;
use webqa_core::config::interview::Interview;
use webqa_core::config::persona::{self, Persona};
use webqa_core::config::purpose::{self, Purpose};
use webqa_core::event::Mode;
use webqa_core::redact::Redactor;

use super::Ran;
use crate::clock::SceneClock;
use crate::fakes::{FakeBrowser, MemoryStore, VIEWPORT};
use crate::interruption::SceneInterrupt;
use crate::judge::SceneJudge;
use crate::launch::SceneLaunch;
use crate::page::PageSpec;
use crate::scene::{PAGES, Scene, found};

/// The persona: from its document when the scene has it as text, else as
/// set. The sentence of an error does not carry the text.
fn persona_of(scene: &Scene) -> Result<Persona, String> {
    let read = |text: &str| persona::parse(text).map_err(|error| error.to_string());
    scene
        .persona_text
        .as_deref()
        .map_or_else(|| Ok(scene.persona.clone()), read)
}

/// The purpose, as [`persona_of`] gives the persona.
fn purpose_of(scene: &Scene) -> Result<Purpose, String> {
    let read = |text: &str| purpose::parse(text).map_err(|error| error.to_string());
    scene
        .purpose_text
        .as_deref()
        .map_or_else(|| Ok(scene.purpose.clone()), read)
}

/// The settings the scene gives. The first document that does not read
/// stops the run.
fn read_settings(scene: &Scene) -> Result<Interview, String> {
    Ok(scene.settings(persona_of(scene)?, purpose_of(scene)?))
}

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

/// The plan of the loop for `scene`.
fn plan(scene: &Scene, settings: Interview, redactor: &Redactor) -> Plan {
    let url = scene.start.build(&scene.origin()).screen.url;
    Plan {
        interview: settings,
        rules: scene.rules(),
        start: redactor.url(&url),
        viewport: VIEWPORT,
        previous: scene.previous.clone(),
        max_duration: scene.max_duration,
        env: Vec::new(),
        withheld: Vec::new(),
        mode: Mode::Scenario,
        started_at: Some("2026-10-18T02:00:00Z".to_owned()),
        detect: Detect::default(),
    }
}

/// The fakes of a run, tied to one gate: whichever waits forever wakes the
/// interrupt once.
struct Fakes {
    judge: SceneJudge,
    clock: SceneClock,
    launch: SceneLaunch,
    interrupt: SceneInterrupt,
    store: MemoryStore,
}

impl Fakes {
    fn of(scene: &Scene) -> Self {
        let gate = Arc::new(Notify::new());
        let now = Arc::new(AtomicU64::new(0));
        let replies = scene.replies.clone();
        let mut judge = SceneJudge::new(replies, Arc::clone(&gate), Arc::clone(&now), scene.think);
        judge.images = scene.images;
        Self {
            judge,
            clock: SceneClock {
                now,
                gate: Arc::clone(&gate),
                ..SceneClock::default()
            },
            launch: SceneLaunch {
                browsers: browsers(scene),
                ..SceneLaunch::default()
            },
            interrupt: SceneInterrupt {
                fire_on: scene.fire_on.clone(),
                gate,
                ..SceneInterrupt::default()
            },
            store: MemoryStore::default(),
        }
    }

    /// Runs the loop with `plan`.
    async fn conduct(&mut self, plan: &Plan, redactor: &Redactor) -> Ending {
        interview(
            &mut self.launch,
            &mut self.judge,
            &mut self.clock,
            &mut self.store,
            redactor,
            &mut self.interrupt,
            plan,
        )
        .await
    }

    fn into_ran(self, ending: Option<Ending>, config_error: Option<String>) -> Ran {
        let touches = {
            let all = self.launch.touches.lock();
            all.unwrap_or_else(PoisonError::into_inner).clone()
        };
        Ran {
            code: ending
                .as_ref()
                .map_or(exit::CONFIG_ERROR, exit::interview_code),
            ending,
            config_error,
            judge: self.judge,
            touches,
            launched: self.launch.launched,
            store: self.store,
            clock: self.clock,
        }
    }
}

/// Runs `scene` once, with fakes.
pub async fn play(scene: &Scene) -> Ran {
    let mut fakes = Fakes::of(scene);
    let redactor = found(
        Redactor::new(&scene.redact).ok(),
        "the redaction rules read",
    );
    match read_settings(scene) {
        Ok(settings) => {
            let ended = fakes
                .conduct(&plan(scene, settings, &redactor), &redactor)
                .await;
            fakes.into_ran(Some(ended), None)
        }
        Err(error) => fakes.into_ran(None, Some(error)),
    }
}
