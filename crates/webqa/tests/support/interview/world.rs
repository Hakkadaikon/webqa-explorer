//! The state of an interview scenario: what the Given steps set up, and the
//! one run a When or a Then asks for.

use serde_json::Value;
use webqa::interview::trials::Ending;
use webqa_core::effect::Touch;
use webqa_core::interview::ledger::Ledger;

use crate::clock::SceneClock;
use crate::fakes::MemoryStore;
use crate::judge::SceneJudge;
use crate::ledger::read_back;
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
    /// The fake judge, with what it was asked.
    pub judge: SceneJudge,
    /// The operations of the browsers with the number of the trial.
    pub touches: Vec<(usize, Touch)>,
    /// How many browsers were opened.
    pub launched: usize,
    /// What the run stored: the event lines, the ledger, the report and the
    /// screenshots.
    pub store: MemoryStore,
    /// The fake clock.
    pub clock: SceneClock,
}

impl InterviewWorld {
    /// Runs the scene once, if it has not run. Later calls do nothing.
    pub async fn run(&mut self) {
        if self.ran.is_none() {
            self.ran = Some(run::play(&self.scene).await);
        }
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
