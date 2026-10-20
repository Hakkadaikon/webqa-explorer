//! What the Given steps of an interview scenario set up. Nothing runs until
//! a When or a Then needs it.

use std::collections::{BTreeSet, VecDeque};
use std::ffi::OsString;
use std::future::Future;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::time::Duration;
use std::{io, iter};

use serde::Serialize;
use serde_json::json;
use webqa_core::config::interview::Reset;
use webqa_core::config::persona::Persona;
use webqa_core::config::purpose::Purpose;
use webqa_core::config::raw::Op;
use webqa_core::guard::domain::AllowedOrigin;
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

/// Where the configuration says the persona document is.
pub const PERSONA_PATH: &str = "persona.yaml";
/// Where the configuration says the purpose document is.
pub const PURPOSE_PATH: &str = "purpose.yaml";

/// How many pages a trial's browser shows at most: the last one stays.
pub const PAGES: usize = 30;

/// The first item of `items`. Nothing there fails the scenario with `what`.
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

/// `n` as a limit. Zero fails the scenario: the configuration has already
/// rejected it.
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
    /// Steps without progress that count as stuck, in `limits`.
    pub stuck_threshold: u32,
    /// The stuck threshold of the `interview` section. `limits` when `None`.
    pub interview_stuck: Option<u32>,
    /// Steps that leave the screen unchanged before the run is frozen.
    pub freeze_threshold: u32,
    /// The most calls of the judge a trial makes for its steps.
    pub max_decisions: u32,
    /// The most operations in a window, in `limits`.
    pub requests_per_hour: u32,
    /// The most operations in a window of the `interview` section. `limits`
    /// when `None`.
    pub interview_requests: Option<u32>,
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
    /// The text of the ledger of the run before, as the file holds it.
    pub previous: Option<String>,
    /// The environment variables the reset command may get.
    pub env: Vec<(OsString, OsString)>,
    /// The strings that must not reach the reset command.
    pub withheld: Vec<String>,
    /// The trial a step means by "the trial", from 1.
    pub trial: usize,
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
            interview_stuck: None,
            freeze_threshold: 5,
            max_decisions: 20,
            requests_per_hour: 100,
            interview_requests: None,
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
            env: Vec::new(),
            withheld: Vec::new(),
            trial: 1,
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

    /// The configuration the CLI would read: the background's origin, the
    /// limits, and the `interview` section of the scene.
    pub fn yaml(&self) -> String {
        let url = json!(self.start.build(&self.origin()).screen.url.as_str());
        let origins = json!(self.origins);
        let base = format!(
            "target:\n  url: {url}\n  allowed_origins: {origins}\n\
             judge:\n  kind: command\n  command: [\"./judge\"]\n  timeout_secs: 30\n  \
             pricing: {{ input_per_1k: 0.0, output_per_1k: 0.0 }}\n\
             limits:\n  freeze_threshold: {}\n  stuck_threshold: {}\n  max_decisions: {}\n  \
             requests_per_hour: {}\n  page_timeout_secs: 10\n  back_confidence: 0.7\n  \
             error_confidence: 0.8\n\
             guard:\n  forbidden: {{ keywords: [], selectors: [] }}\n  \
             allowed: {{ selectors: [] }}\n  leave_paywall: back\n\
             interview:\n  persona: {PERSONA_PATH}\n  trials: {}\n  hesitation_threshold: {:?}\n  \
             confidence_threshold: {:?}\n  max_natural_questions: {}\n",
            self.freeze_threshold,
            self.stuck_threshold,
            self.max_decisions,
            self.requests_per_hour,
            self.trials,
            self.hesitation,
            self.confidence,
            self.max_natural,
        );
        let mut yaml = base;
        yaml.push_str(&self.optional_yaml());
        yaml
    }

    /// The lines of the `interview` section that the scene may leave out.
    fn optional_yaml(&self) -> String {
        let purpose = self.purpose_text.as_ref().map(|_| PURPOSE_PATH);
        [
            entry("purpose", purpose),
            entry("stuck_threshold", self.interview_stuck),
            entry("requests_per_hour", self.interview_requests),
            entry("reset", self.reset.as_ref().map(reset_json)),
        ]
        .concat()
    }

    /// The document at `path`: the persona as text or as set, and the
    /// purpose as text.
    ///
    /// # Errors
    ///
    /// Not found for any other path.
    pub fn document(&self, path: &Path) -> io::Result<String> {
        let persona = &self.persona;
        let set = json!({
            "name": persona.name,
            "profile": persona.profile,
            "it_literacy": persona.it_literacy,
            "wants": persona.wants,
            "avoids": persona.avoids,
        });
        let text = if path == Path::new(PERSONA_PATH) {
            Some(self.persona_text.clone().unwrap_or_else(|| set.to_string()))
        } else if path == Path::new(PURPOSE_PATH) {
            self.purpose_text.clone()
        } else {
            None
        };
        text.ok_or_else(|| io::ErrorKind::NotFound.into())
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

/// The `reset` section of `reset`, in JSON, which YAML reads.
fn reset_json(reset: &Reset) -> serde_json::Value {
    let command: Vec<&String> = iter::once(&reset.program).chain(&reset.args).collect();
    json!({ "command": command, "timeout_secs": reset.timeout_secs.get() })
}

/// The line `key: value` of the `interview` section, in JSON, which YAML
/// reads. Nothing when there is no value.
fn entry(key: &str, value: Option<impl Serialize>) -> String {
    value.map_or_else(String::new, |value| format!("  {key}: {}\n", json!(value)))
}
