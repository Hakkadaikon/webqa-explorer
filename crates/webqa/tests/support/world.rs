//! The state of a scenario: what the Given steps set up, and the one run a
//! When or a Then asks for.

use std::collections::BTreeSet;
use std::time::Duration;

use run::play;
use serde_json::Value;
use webqa::run::session::Ending;
use webqa_core::effect::Touch;
use webqa_core::machine::{self, ExitReason, Facts, Heard, Outcome, State, Step};
use webqa_core::ports::Clock;

use crate::browser::SceneBrowser;
use crate::clock::SceneClock;
use crate::fakes::MemoryStore;
use crate::interruption::SceneInterrupt;
use crate::judge::SceneJudge;
use crate::scene::{Scene, found};

/// One scenario's scene, its run, and the step poked after it.
#[derive(Debug, Default, cucumber::World)]
pub struct QaWorld {
    /// What the Given steps set up.
    pub scene: Scene,
    /// What the run left.
    pub ran: Option<Ran>,
    /// The result of one more step given to the ended state.
    pub poked: Option<Step>,
}

/// What one run left.
#[derive(Debug)]
pub struct Ran {
    /// How the run ended. `None` when the configuration stopped it.
    pub ending: Option<Ending>,
    /// The exit code.
    pub code: u8,
    /// The sentence of the configuration error.
    pub config_error: Option<String>,
    /// The fake judge.
    pub judge: SceneJudge,
    /// The fake browser.
    pub browser: SceneBrowser,
    /// The fake clock.
    pub clock: SceneClock,
    /// What the run stored.
    pub store: MemoryStore,
    /// The fake interrupt.
    pub interrupt: SceneInterrupt,
    /// The questions the Given steps queued.
    pub given_steps: u64,
}

/// The names of the elements marked operated in a memory file.
fn operated(file: &Value) -> BTreeSet<String> {
    let screens = file.get("screens").and_then(Value::as_object);
    let screens = screens.into_iter().flat_map(|screens| screens.values());
    let lists = screens.filter_map(|screen| screen.get("elements"));
    let entries = lists.filter_map(Value::as_array).flatten();
    let done = entries.filter(|entry| entry.get("operated") == Some(&Value::Bool(true)));
    let names = done.filter_map(|entry| entry.get("name").and_then(Value::as_str));
    names.map(str::to_owned).collect()
}

/// The point a run has reached, read from the lines of its event log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Beat {
    /// The judge was asked.
    Ask,
    /// The core operated the browser on its own.
    Core,
    /// The scenario moved on to the next stage.
    Advance,
    /// The run ended.
    End,
}

/// The beat a line of the log of this `kind` is, if it is one.
fn beat_of(kind: &str) -> Option<Beat> {
    let beats = [
        ("asked", Beat::Ask),
        ("recovered", Beat::Core),
        ("stage_advanced", Beat::Advance),
        ("run_finished", Beat::End),
    ];
    beats
        .iter()
        .find(|(name, _)| *name == kind)
        .map(|&(_, beat)| beat)
}

impl QaWorld {
    /// Runs the scene once, if it has not run. Later calls do nothing.
    pub async fn run(&mut self) {
        if self.ran.is_none() {
            self.ran = Some(play(&self.scene).await);
        }
    }

    /// The run, after `run`. Fails the scenario when it did not run.
    pub fn ran(&self) -> &Ran {
        found(self.ran.as_ref(), "the scenario ran the scene")
    }

    /// The outcome of a run that ended itself or by the user.
    fn outcome(&self) -> Option<&Outcome> {
        let ending = self.ran().ending.as_ref();
        ending.and_then(|ending| match ending {
            Ending::Finished { outcome, .. } => Some(outcome),
            Ending::Abandoned | Ending::Failed(_) => None,
        })
    }

    /// Why the run ended. `None` when the harness stopped it (the judge
    /// ran out of replies, or something waited forever) and the user did
    /// not interrupt: the run "has not ended".
    pub fn reason(&self) -> Option<ExitReason> {
        let reason = self.outcome()?.reason;
        let by_harness = reason == ExitReason::UserAborted && !self.scene.user_interrupts;
        (!by_harness).then_some(reason)
    }

    /// The last state: the outcome's. A run with no outcome fails the
    /// scenario.
    pub fn state(&self) -> &State {
        &found(self.outcome(), "the run has an outcome").state
    }

    /// Calls of the judge, the one left waiting included.
    pub fn asked(&self) -> usize {
        self.ran().judge.asked
    }

    /// Replies the judge gave (answers and failures).
    pub fn answered(&self) -> usize {
        self.ran().judge.answered
    }

    /// The times the clock was asked to sleep until.
    pub fn sleeps(&self) -> &[Duration] {
        &self.ran().clock.sleeps
    }

    /// The beats of the run in order. A run the harness stopped while the
    /// judge waited has one more `Ask`, before its end.
    fn beats(&self) -> Vec<Beat> {
        let kinds = self.kinds();
        let mut beats: Vec<Beat> = kinds.iter().filter_map(|kind| beat_of(kind)).collect();
        let end = beats.iter().rposition(|beat| *beat == Beat::End);
        let stalled = self.ran().judge.stalled;
        let at = end.unwrap_or(beats.len());
        if stalled {
            beats.insert(at, Beat::Ask);
        }
        beats
    }

    /// What the step of the When was: the first beat after the Given ones.
    pub fn when_beat(&self) -> Beat {
        let given = usize::try_from(self.ran().given_steps).unwrap_or(usize::MAX);
        let beats = self.beats();
        let asks = beats
            .iter()
            .enumerate()
            .filter(|(_, beat)| **beat == Beat::Ask);
        let given_at: Vec<usize> = asks.map(|(at, _)| at).take(given).collect();
        assert_eq!(given_at.len(), given, "the Given steps ran");
        let skip = given_at.last().map_or(0, |at| at + 1);
        beats.get(skip).copied().unwrap_or(Beat::End)
    }

    /// Whether the step of the When asked the judge.
    pub fn when_asked(&self) -> bool {
        self.when_beat() == Beat::Ask
    }

    /// The browser operations as (op, target name).
    pub fn touched(&self) -> Vec<(String, Option<String>)> {
        let touches = self.ran().browser.touches.iter();
        let named = |t: &Touch| (t.op.clone(), t.target.as_ref().map(|id| id.name.clone()));
        touches.map(named).collect()
    }

    /// The names of the elements operated in the memory, every screen.
    pub fn explored(&self) -> BTreeSet<String> {
        let memory = self.ran().store.memories.get(&self.scene.origin());
        let file: Value = memory
            .and_then(|text| serde_json::from_str(text).ok())
            .unwrap_or_default();
        operated(&file)
    }

    /// The lines of the event log.
    pub fn lines(&self) -> Vec<Value> {
        self.ran().store.lines.clone()
    }

    /// The `kind` of each line.
    pub fn kinds(&self) -> Vec<String> {
        let kind = |line: &Value| {
            let kind = line.get("kind").and_then(Value::as_str);
            kind.unwrap_or_default().to_owned()
        };
        self.lines().iter().map(kind).collect()
    }

    /// The report, once written.
    pub fn report(&self) -> Option<&Value> {
        self.ran().store.report.as_ref()
    }

    /// The time window's count at the end.
    pub fn window_used(&self) -> u32 {
        self.state().window().used(self.ran().clock.elapsed())
    }

    /// Gives the ended state one more step at `later` after its end.
    pub fn poke(&mut self, later: Duration) {
        let ended = self.ran().clock.elapsed();
        let state = self.state().clone();
        let facts = Facts::Outside(Heard::Woke);
        self.poked = Some(machine::step(state, facts, ended + later));
    }
}

#[path = "world_run.rs"]
mod run;

#[path = "world_checks.rs"]
pub mod checks;
