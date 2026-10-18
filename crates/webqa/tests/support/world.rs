//! The state of a scenario: the fake ports and what the scenario set up.

use std::collections::BTreeMap;

use url::Url;
use webqa::run::{Ports, RunError, explore};
use webqa_core::machine::Outcome;
use webqa_core::ports::Observation;
use webqa_core::redact::Redactor;

use crate::fakes::{self, FakeBrowser, FakeClock, MemoryStore, RecordingJudge};

/// The operation kinds offered to the judge model.
const OPS: [&str; 8] = [
    "click",
    "input",
    "scroll_down",
    "scroll_up",
    "back",
    "wait",
    "done",
    "stuck",
];

/// One scenario's fakes and settings.
#[derive(Debug, Default, cucumber::World)]
pub struct QaWorld {
    /// The fake browser.
    pub browser: FakeBrowser,
    /// The stub judge model.
    pub judge: RecordingJudge,
    /// The fake clock.
    pub clock: FakeClock,
    /// The store in memory.
    pub store: MemoryStore,
    /// Masks nothing until a scenario names a secret.
    pub redactor: Redactor,
    /// The URL of the page the run starts on.
    pub origins: Vec<String>,
    /// The (role, name) elements of the page.
    pub elements: Vec<(String, String)>,
    /// Limits from the background. The provisional step does not use them yet.
    pub limits: BTreeMap<&'static str, u32>,
}

impl QaWorld {
    /// Shows the page, unchanged by the touch, and runs the loop.
    pub async fn run(&mut self) -> Result<Outcome, RunError> {
        let pages = self.pages();
        self.browser
            .pages
            .extend(pages.iter().chain(&pages).cloned());
        let mut ports = Ports::new(
            &mut self.browser,
            &mut self.judge,
            &mut self.clock,
            &mut self.store,
            &self.redactor,
        );
        let ops = OPS.map(str::to_owned).to_vec();
        explore(&mut ports, ops).await
    }

    /// The page at each origin. A URL that cannot be read gives no page, and
    /// the run fails.
    fn pages(&self) -> Vec<Observation> {
        let elements = self.elements();
        let urls = self.origins.iter().flat_map(|origin| Url::parse(origin));
        urls.map(|url| fakes::page(&url, &elements)).collect()
    }

    fn elements(&self) -> Vec<(&str, &str)> {
        let elements = self.elements.iter();
        elements
            .map(|(role, name)| (role.as_str(), name.as_str()))
            .collect()
    }
}
