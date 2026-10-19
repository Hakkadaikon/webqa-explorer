//! The state of a scenario: the fake ports and what the scenario set up.

use std::collections::BTreeMap;
use std::future;
use std::iter;
use std::num::NonZeroU32;

use url::Url;
use webqa::interrupt::Interrupt;
use webqa::run::{Ports, RunError, explore};
use webqa_core::config::raw::Op;
use webqa_core::explore::Memory;
use webqa_core::guard::domain::AllowedOrigin;
use webqa_core::machine::{Outcome, Rules};
use webqa_core::ports::{Observation, Pending};
use webqa_core::redact::Redactor;

use crate::fakes::{self, FakeBrowser, FakeClock, MemoryStore, RecordingJudge};

/// The operation kinds offered to the judge model: all of them.
const OPS: [Op; 8] = [
    Op::Click,
    Op::Input,
    Op::ScrollDown,
    Op::ScrollUp,
    Op::Back,
    Op::Wait,
    Op::Done,
    Op::Stuck,
];

/// How many times each page is observed: at the start, when the element is
/// looked up again, and after the operation.
const OBSERVED: usize = 3;

/// The user who never interrupts.
struct Never;

impl Interrupt for Never {
    fn wait(&mut self) -> Pending<'_, ()> {
        Box::pin(future::pending())
    }
}

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
    /// Limits from the background.
    pub limits: BTreeMap<&'static str, u32>,
}

impl QaWorld {
    /// Shows the page, unchanged by the touch, and runs the loop.
    pub async fn run(&mut self) -> Result<Outcome, RunError> {
        let pages = self.pages();
        let observed = pages.iter().flat_map(|page| iter::repeat_n(page, OBSERVED));
        self.browser.pages.extend(observed.cloned());
        let rules = self.rules();
        let mut ports = Ports::new(
            &mut self.browser,
            &mut self.judge,
            &mut self.clock,
            &mut self.store,
            &self.redactor,
        );
        explore(&mut ports, &mut Never, rules, Memory::default()).await
    }

    /// The rules of the background. An origin that cannot be read fails the
    /// scenario.
    fn rules(&self) -> Rules {
        let limits = [
            "freeze_threshold",
            "stuck_threshold",
            "max_decisions",
            "requests_per_hour",
        ]
        .map(|name| self.limit(name));
        let origins = self.origins.iter();
        let parsed: Vec<_> = origins.map(|o| AllowedOrigin::parse(o)).collect();
        let readable = parsed.iter().all(Result::is_ok);
        assert!(readable, "the origins of the background read: {parsed:?}");
        let allowed = parsed.into_iter().flatten().collect();
        fakes::rules(&OPS, limits, allowed)
    }

    /// A limit of the background. A limit not set, or zero, fails the scenario.
    fn limit(&self, name: &str) -> NonZeroU32 {
        let limit = self.limits.get(name).copied().and_then(NonZeroU32::new);
        assert!(limit.is_some(), "the background sets {name} above zero");
        limit.unwrap_or(NonZeroU32::MIN)
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
