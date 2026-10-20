//! Runs a scene once, with fakes tied to one gate.

use std::collections::VecDeque;
use std::fmt::Display;
use std::iter;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, PoisonError};

use tokio::sync::Notify;
use webqa::exit;
use webqa::interview::trials::{Ending, Plan, interview};
use webqa::run::Detect;
use webqa::setup;
use webqa_core::config::interview::Interview;
use webqa_core::config::purpose::Aim;
use webqa_core::event::Mode;
use webqa_core::interview::compare::check_previous;
use webqa_core::interview::ledger::{self, Goal, Ledger};
use webqa_core::redact::Redactor;

use super::Ran;
use crate::clock::SceneClock;
use crate::fakes::{FakeBrowser, MemoryStore, VIEWPORT};
use crate::interruption::SceneInterrupt;
use crate::judge::SceneJudge;
use crate::launch::SceneLaunch;
use crate::page::PageSpec;
use crate::scene::{PAGES, Scene, found};

/// The stages of starting, in the order of the CLI: the configuration
/// with its documents, then the ledger before and whether its head is of
/// this persona and purpose. The first failure stops the run with its
/// sentence, which carries no text of the documents.
fn start(scene: &Scene, url: &str) -> Result<(Interview, Option<Ledger>), String> {
    let interview = configured(scene)?;
    let previous = scene
        .previous
        .as_deref()
        .map(|text| previous(text, &interview, url));
    Ok((interview, previous.transpose()?))
}

/// The settings and the documents, read as the CLI reads them. A purpose
/// set without its text stands for the document.
fn configured(scene: &Scene) -> Result<Interview, String> {
    let read = |path: &Path| scene.document(path);
    let loaded = setup::load_interview(&scene.yaml(), &|_| None, &read);
    let (_, mut interview) = loaded.map_err(sentence)?;
    if scene.purpose_text.is_none() {
        interview.purpose = scene.purpose.clone();
    }
    Ok(interview)
}

/// The ledger before, read and checked as the CLI does.
fn previous(text: &str, interview: &Interview, url: &str) -> Result<Ledger, String> {
    let ledger = ledger::read(text).map_err(sentence)?;
    let goal = goal_of(&interview.purpose.aim);
    let checked = check_previous(&ledger.head, &interview.persona.name, &goal, url);
    checked.map(|()| ledger).map_err(sentence)
}

/// The purpose a ledger head is checked against, as the CLI has it.
fn goal_of(aim: &Aim) -> Goal {
    match aim {
        Aim::Written(written) => Goal::Written(written.goal.clone()),
        Aim::Inferred => Goal::Inferred(Vec::new()),
    }
}

/// The sentence of an error.
fn sentence(error: impl Display) -> String {
    error.to_string()
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
fn plan(scene: &Scene, started: (Interview, Option<Ledger>), redactor: &Redactor) -> Plan {
    let url = scene.start.build(&scene.origin()).screen.url;
    let (interview, previous) = started;
    Plan {
        interview,
        rules: scene.rules(),
        start: redactor.url(&url),
        viewport: VIEWPORT,
        previous,
        max_duration: scene.max_duration,
        env: scene.env.clone(),
        withheld: scene.withheld.clone(),
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
    let url = scene.start.build(&scene.origin()).screen.url;
    match start(scene, redactor.url(&url).as_inner()) {
        Ok(started) => {
            let ended = fakes
                .conduct(&plan(scene, started, &redactor), &redactor)
                .await;
            fakes.into_ran(Some(ended), None)
        }
        Err(error) => fakes.into_ran(None, Some(error)),
    }
}
