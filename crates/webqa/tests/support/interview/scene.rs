//! What the Given steps of an interview scenario set up. Nothing runs until
//! a When or a Then needs it.

use std::collections::{BTreeSet, VecDeque};
use std::future::Future;
use std::num::{NonZeroU32, NonZeroU64};
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use webqa_core::config::interview::{Interview, Reset};
use webqa_core::config::persona::Persona;
use webqa_core::config::purpose::Purpose;
use webqa_core::config::raw::Op;
use webqa_core::guard::domain::AllowedOrigin;
use webqa_core::interview::ledger::Ledger;
use webqa_core::judge::{Answers, JudgeError};
use webqa_core::machine::Rules;
use webqa_core::redact::Rule;

use crate::fakes;
use crate::ledger::Aliases;
use crate::page::PageSpec;

/// The name of the rule that hides e-mail addresses.
pub const EMAIL_RULE: &str = "email";
/// What an e-mail address looks like.
const EMAIL: &str = r"[\w.+-]+@[\w-]+\.[\w.]+";

/// The directory of the scripts of the reset command.
const SCRIPTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reset/");

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

/// How many pages a trial's browser shows at most: the last one stays.
pub const PAGES: usize = 30;

/// The only item of `items`. Nothing there fails the scenario with `what`.
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

/// `n` as a limit. Zero fails the scenario: the scene names its limits.
pub fn limit(n: u32, name: &str) -> NonZeroU32 {
    found(NonZeroU32::new(n), &format!("{name} is above zero"))
}

/// What the Given steps set up.
#[derive(Debug)]
pub struct Scene {
    /// The allowed origins of the background.
    pub origins: Vec<String>,
    /// The persona, when the document is not given as text.
    pub persona: Persona,
    /// The text of the persona document, to be read at the start.
    pub persona_text: Option<String>,
    /// The purpose, when the document is not given as text.
    pub purpose: Purpose,
    /// The text of the purpose document, to be read at the start.
    pub purpose_text: Option<String>,
    /// How many trials to run.
    pub trials: u32,
    /// The threshold of hesitation.
    pub hesitation: f64,
    /// The confidence below which a step is hesitant.
    pub confidence: f64,
    /// Steps without progress that count as stuck.
    pub stuck_threshold: u32,
    /// Steps that leave the screen unchanged before the run is frozen.
    pub freeze_threshold: u32,
    /// The most calls of the judge a trial makes for its steps.
    pub max_decisions: u32,
    /// The most operations in a window.
    pub requests_per_hour: u32,
    /// The most questions about naturalness.
    pub max_natural: u32,
    /// The time limit from the start of the run.
    pub max_duration: Option<Duration>,
    /// The command that resets the service before each trial.
    pub reset: Option<Reset>,
    /// The words the redaction hides.
    pub redact: Vec<Rule>,
    /// Whether the judge reads images.
    pub images: bool,
    /// The page every trial starts on.
    pub start: PageSpec,
    /// The pages after the start page, trial by trial. The last page of a
    /// trial stays.
    pub after: Vec<Vec<PageSpec>>,
    /// The replies of the judge, in order.
    pub replies: VecDeque<Result<Answers, JudgeError>>,
    /// How long each call of the judge takes.
    pub think: Duration,
    /// The calls of the interrupt that fire, from 1.
    pub fire_on: BTreeSet<usize>,
    /// The ledger of the run before.
    pub previous: Option<Ledger>,
    /// The numbers the examples of the feature stand for.
    pub aliases: Aliases,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            origins: Vec::new(),
            persona: Persona {
                name: "名前".to_owned(),
                profile: "利用者".to_owned(),
                it_literacy: "ふつう".to_owned(),
                wants: "目的を果たす".to_owned(),
                avoids: vec!["しないこと".to_owned()],
            },
            persona_text: None,
            purpose: Purpose::inferred(),
            purpose_text: None,
            trials: 1,
            hesitation: 0.5,
            confidence: 0.5,
            stuck_threshold: 5,
            freeze_threshold: 5,
            max_decisions: 20,
            requests_per_hour: 100,
            max_natural: 10,
            max_duration: None,
            reset: None,
            redact: Vec::new(),
            images: false,
            start: PageSpec::at("/"),
            after: Vec::new(),
            replies: VecDeque::new(),
            think: Duration::ZERO,
            fire_on: BTreeSet::new(),
            previous: None,
            aliases: Aliases::default(),
        }
    }
}

impl Scene {
    /// The rules of the scene. An origin that cannot be read fails the
    /// scenario.
    pub fn rules(&self) -> Rules {
        let parsed: Vec<_> = self
            .origins
            .iter()
            .map(|origin| AllowedOrigin::parse(origin))
            .collect();
        assert!(
            parsed.iter().all(Result::is_ok),
            "the origins of the background read: {parsed:?}"
        );
        let limits = [
            (self.freeze_threshold, "freeze_threshold"),
            (self.stuck_threshold, "stuck_threshold"),
            (self.max_decisions, "max_decisions"),
            (self.requests_per_hour, "requests_per_hour"),
        ]
        .map(|(n, name)| limit(n, name));
        Rules {
            max_duration: self.max_duration,
            ..fakes::rules(&OPS, limits, parsed.into_iter().flatten().collect())
        }
    }

    /// The settings of the interview, with the persona and the purpose
    /// read.
    pub fn settings(&self, persona: Persona, purpose: Purpose) -> Interview {
        Interview {
            persona,
            purpose,
            trials: limit(self.trials, "trials"),
            hesitation_threshold: self.hesitation,
            confidence_threshold: self.confidence,
            stuck_threshold: limit(self.stuck_threshold, "stuck_threshold"),
            requests_per_hour: limit(self.requests_per_hour, "requests_per_hour"),
            max_natural_questions: limit(self.max_natural, "max_natural"),
            reset: self.reset.clone(),
        }
    }

    /// The first allowed origin, such as `https://shop.example.test`.
    pub fn origin(&self) -> String {
        let first = found(self.origins.first(), "the background names an origin");
        first.trim_end_matches('/').to_owned()
    }

    /// Hides e-mail addresses.
    pub fn hide_emails(&mut self) {
        self.redact.push(Rule::pattern(EMAIL_RULE, EMAIL));
    }

    /// Resets the service with the script `name` of the fixtures, which
    /// may take `timeout_secs`.
    pub fn reset_with(&mut self, name: &str, timeout_secs: u64) {
        self.reset = Some(Reset {
            program: format!("{SCRIPTS}{name}"),
            args: Vec::new(),
            timeout_secs: found(NonZeroU64::new(timeout_secs), "the timeout is above zero"),
        });
    }
}
