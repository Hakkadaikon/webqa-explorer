//! The state of an interview scenario: what the Given steps set up, and the
//! one run a When or a Then asks for.

use std::collections::BTreeMap;

use serde_json::Value;
use webqa::interview::trials::Ending;
use webqa_core::effect::Touch;
use webqa_core::interview::ledger::Ledger;

use crate::fakes::MemoryStore;
use crate::judge::SceneJudge;
use crate::launch::Opened;
use crate::ledger::read_back;
use crate::run_clock::TrialClock;
use crate::scene::{Scene, found};

#[path = "world_run.rs"]
mod run;

/// One scenario's scene and its run.
#[derive(Debug, Default, cucumber::World)]
pub struct InterviewWorld {
    /// What the Given steps set up.
    pub scene: Scene,
    /// What the run left.
    pub ran: Option<Ran>,
    /// What the run before left, when the scenario runs again.
    pub before: Option<Ran>,
    /// What a step hands to the steps after it, by name. Not given to the run.
    pub kept: BTreeMap<&'static str, String>,
}

/// What one run left.
#[derive(Debug)]
pub struct Ran {
    /// How the interview ended. `None` when the configuration stopped it.
    pub ending: Option<Ending>,
    /// The exit code.
    pub code: u8,
    /// The sentence of the configuration error.
    pub config_error: Option<String>,
    /// The fake judge, with what it was asked. Asked nothing when the run
    /// went through the bridge.
    pub judge: SceneJudge,
    /// The request lines the bridge read, when the run went through it.
    pub bridge: Vec<Value>,
    /// The operations of the browsers with the number of the trial.
    pub touches: Vec<(usize, Touch)>,
    /// What happened as each browser opened, with the number of the trial.
    pub openings: Vec<(usize, Opened)>,
    /// How many browsers were opened.
    pub launched: usize,
    /// What the run stored: the event lines, the ledger, the report and the
    /// screenshots.
    pub store: MemoryStore,
    /// The fake clock.
    pub clock: TrialClock,
}

impl InterviewWorld {
    /// Runs the scene once, if it has not run. Later calls do nothing.
    pub async fn run(&mut self) {
        if self.ran.is_none() {
            self.ran = Some(run::play(&self.scene).await);
        }
    }

    /// Keeps the run as the run before, so that the next `run` runs the
    /// scene again. Steps may change the scene in between.
    pub fn again(&mut self) {
        self.before = self.ran.take();
    }

    /// The run before, after `again`. Fails the scenario when there is none.
    pub fn before(&self) -> &Ran {
        found(self.before.as_ref(), "the scenario ran the scene before")
    }

    /// The run, after `run`. Fails the scenario when it did not run.
    pub fn ran(&self) -> &Ran {
        found(self.ran.as_ref(), "the scenario ran the scene")
    }

    /// The ledger the run wrote, read back with the reader of the product.
    /// `None` when none was written.
    pub fn ledger(&self) -> Option<Ledger> {
        let text = self.ran().store.ledger.as_deref()?;
        Some(read_back(text))
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
}
