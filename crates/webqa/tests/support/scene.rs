//! What the Given steps set up. Nothing runs until a When or Then needs it.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::future::{Future, ready};
use std::num::NonZeroU32;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use url::Url;
use webqa_core::config::raw::{Op, Stage};
use webqa_core::event::Mode;
use webqa_core::guard::domain::AllowedOrigin;
use webqa_core::guard::forbidden::Forbidden;
use webqa_core::machine::Rules;
use webqa_core::redact::Rule;

use crate::fakes;
use crate::judge::Reply;
use crate::page::PageSpec;
use crate::store::Hold;

/// The operation kinds offered to the judge model: all of them.
const OPS: [Op; 9] = [
    Op::Click,
    Op::Input,
    Op::ScrollDown,
    Op::ScrollUp,
    Op::Back,
    Op::Enter,
    Op::Wait,
    Op::Done,
    Op::Stuck,
];

/// The limits of the background that the rules need, in the order of
/// `fakes::Limits`.
const LIMITS: [&str; 4] = [
    "freeze_threshold",
    "stuck_threshold",
    "max_decisions",
    "requests_per_hour",
];

/// The only item of `items`. Nothing there fails the scenario with `what`.
///
/// The scenarios name what they look for, so a miss is a mistake of the
/// scene.
pub fn found<T>(items: impl IntoIterator<Item = T>, what: &str) -> T {
    let mut items: Vec<T> = items.into_iter().collect();
    assert!(!items.is_empty(), "{what}");
    items.swap_remove(0)
}

/// Polls `pending` once, without a waker that does anything.
pub fn poll_once<F: Future + Unpin>(pending: &mut F) -> Poll<F::Output> {
    let mut cx = Context::from_waker(Waker::noop());
    Pin::new(pending).poll(&mut cx)
}

/// What waits forever in a scene.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Stall {
    /// Nothing: every fake answers.
    #[default]
    Nothing,
    /// The page does not answer an operation.
    Touch,
    /// The wait of the window never ends.
    Sleep,
}

/// What the Given steps set up. Nothing runs until a When or Then needs it.
#[derive(Debug)]
pub struct Scene {
    /// The kind of run.
    pub mode: Mode,
    /// The allowed origins of the background.
    pub origins: Vec<String>,
    /// The limits of the background and the Given steps.
    pub limits: BTreeMap<&'static str, u32>,
    /// The operation kinds offered to the judge.
    pub ops: Vec<Op>,
    /// `limits.back_confidence`.
    pub back_confidence: f64,
    /// `limits.error_confidence`.
    pub error_confidence: f64,
    /// `limits.max_duration_secs`.
    pub max_duration: Option<Duration>,
    /// The stages of the scenario.
    pub stages: Vec<Stage>,
    /// The words added to the forbidden ones.
    pub keywords: Vec<String>,
    /// The operation that leaves a paywall.
    pub leave_paywall: String,
    /// Whether the judge is asked about meaning.
    pub probes: bool,
    /// The redaction rules.
    pub redact: Vec<Rule>,
    /// The names of the inputs that have a value to type.
    pub inputs: Vec<String>,
    /// The page the run starts on.
    pub start: PageSpec,
    /// The page after each operation, in order. When they run out, the
    /// page stays.
    pub after: VecDeque<PageSpec>,
    /// The page the next observations show, one per observation.
    pub looks: VecDeque<PageSpec>,
    /// The replies of the judge.
    pub replies: VecDeque<Reply>,
    /// The memory files in the store before the run, by origin.
    pub memories: BTreeMap<String, String>,
    /// The elapsed time the run starts at.
    pub clock_start: Duration,
    /// How long each question to the judge takes.
    pub think: Duration,
    /// The calls of the interrupt that fire, from 1.
    pub fire_on: BTreeSet<usize>,
    /// Whether an interrupt is the user's, the one after waiting forever
    /// included.
    pub user_interrupts: bool,
    /// What never finishes, if anything.
    pub stall: Stall,
    /// `MemoryStore.limit`.
    pub store_limit: Option<usize>,
    /// The browser operation that fails, counted from 1, if one does.
    pub fail_touch: Option<usize>,
    /// Which write of the memory is held.
    pub store_hold: Hold,
    /// The YAML to start from.
    pub yaml: Option<String>,
    /// The environment the configuration is checked with.
    pub env: BTreeMap<String, String>,
    /// What a Rule's steps pass on to the later ones.
    pub marks: BTreeMap<&'static str, u64>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            mode: Mode::Scenario,
            origins: Vec::new(),
            limits: BTreeMap::new(),
            ops: OPS.to_vec(),
            back_confidence: 0.7,
            error_confidence: 0.8,
            max_duration: None,
            stages: Vec::new(),
            keywords: Vec::new(),
            leave_paywall: "back".to_owned(),
            probes: false,
            redact: Vec::new(),
            inputs: Vec::new(),
            start: PageSpec::at("/"),
            after: VecDeque::new(),
            looks: VecDeque::new(),
            replies: VecDeque::new(),
            memories: BTreeMap::new(),
            clock_start: Duration::ZERO,
            think: Duration::ZERO,
            fire_on: BTreeSet::new(),
            user_interrupts: false,
            stall: Stall::Nothing,
            store_limit: None,
            fail_touch: None,
            store_hold: Hold::None,
            yaml: None,
            env: BTreeMap::new(),
            marks: BTreeMap::new(),
        }
    }
}

impl Scene {
    /// The rules of the scene. A limit not set, or zero, or an origin that
    /// cannot be read fails the scenario.
    pub fn rules(&self) -> Rules {
        let parsed: Vec<_> = self
            .origins
            .iter()
            .map(|o| AllowedOrigin::parse(o))
            .collect();
        let readable = parsed.iter().all(Result::is_ok);
        assert!(readable, "the origins of the background read: {parsed:?}");
        let allowed = parsed.into_iter().flatten().collect();
        let limits = LIMITS.map(|name| self.limit(name));
        Rules {
            mode: self.mode,
            max_duration: self.max_duration,
            back_confidence: self.back_confidence,
            forbidden: Forbidden::new(&self.keywords),
            leave_paywall: self.leave_paywall.clone(),
            stages: self.stages.clone(),
            input_names: self.inputs.iter().cloned().collect(),
            ..fakes::rules(&self.ops, limits, allowed)
        }
    }

    /// The questions the Given steps queued: each is one step of the run.
    pub fn given_steps(&self) -> u64 {
        u64::try_from(self.replies.len()).unwrap_or(u64::MAX)
    }

    /// The origin of the first allowed origin, such as
    /// `https://app.example.test`. None or one that cannot be read fails
    /// the scenario.
    pub fn origin(&self) -> String {
        let first = found(self.origins.first(), "the background names an origin");
        let url = found(Url::parse(first).ok(), "the first origin reads");
        url.origin().ascii_serialization()
    }

    /// Makes the start page show a button named `name`, if it does not.
    pub fn show_button(&mut self, name: impl Into<String>) {
        let name = name.into();
        let shown = self.start.elements.iter().any(|e| e.name == name);
        if !shown {
            self.start.elements.push(PageSpec::button(&name));
        }
    }

    /// A limit of the background. A limit not set, or zero, fails the scenario.
    fn limit(&self, name: &str) -> NonZeroU32 {
        let limit = self.limits.get(name).copied().and_then(NonZeroU32::new);
        assert!(limit.is_some(), "the scene sets {name} above zero");
        limit.unwrap_or(NonZeroU32::MIN)
    }
}

/// A scene with the background's origin and limits.
fn background() -> Scene {
    let mut scene = Scene::default();
    scene.origins.push("https://app.example.test".to_owned());
    scene.limits.extend([
        ("freeze_threshold", 3),
        ("stuck_threshold", 4),
        ("max_decisions", 20),
        ("requests_per_hour", 10),
    ]);
    scene
}

/// Checks the scene: the rules, the origin, the buttons and `found`.
pub fn check() {
    let mut scene = background();
    scene.replies.push_back(Reply::op("wait", 0.9));
    let rules = scene.rules();
    let limits = [
        rules.freeze_threshold.get(),
        rules.stuck_threshold.get(),
        rules.max_decisions.get(),
        rules.requests_per_hour.get(),
    ];
    assert_eq!(limits, [3, 4, 20, 10], "the limits of the scene");
    let defaults = (
        rules.mode,
        rules.allowed.len(),
        rules.leave_paywall.as_str(),
    );
    assert_eq!(
        defaults,
        (Mode::Scenario, 1, "back"),
        "the defaults of the rules"
    );
    assert_eq!(scene.given_steps(), 1, "one reply is queued");
    assert_eq!(scene.origin(), "https://app.example.test", "the origin");
    check_buttons(&mut scene);
    assert_eq!(found([7, 8], "two"), 7, "the first of the items");
    let mut pending = ready(1);
    assert_eq!(poll_once(&mut pending), Poll::Ready(1), "ready at once");
}

/// A button is added once.
fn check_buttons(scene: &mut Scene) {
    scene.show_button("保存");
    scene.show_button("保存");
    scene.show_button("ヘルプ");
    let shown = scene.start.elements.iter();
    let names: Vec<&str> = shown.map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["保存", "ヘルプ"], "a button is added once");
}
