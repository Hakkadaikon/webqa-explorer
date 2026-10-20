//! What the Given steps of an interview scenario set up. Nothing runs until
//! a When or a Then needs it.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::OsString;
use std::future::Future;
use std::num::NonZeroU64;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use webqa_core::config::interview::Reset;
use webqa_core::config::persona::Persona;
use webqa_core::config::purpose::Purpose;
use webqa_core::judge::{Answers, JudgeError};
use webqa_core::redact::Rule;

use crate::clock::millis;
use crate::ledger::Aliases;

#[path = "scene_yaml.rs"]
mod yaml;
use crate::page::PageSpec;

/// The name of the rule that hides e-mail addresses.
pub const EMAIL_RULE: &str = "email";
/// What an e-mail address looks like.
const EMAIL: &str = r"[\w.+-]+@[\w-]+\.[\w.]+";

/// The directory of the scripts of the reset command.
const SCRIPTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reset/");

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
    /// The environment variable the judge's API key is read from.
    pub api_key_env: Option<String>,
    /// Whether the configuration names a storage state.
    pub auth: bool,
    /// The keywords of the forbidden elements.
    pub forbidden: Vec<String>,
    /// The bridge the judge runs through, declaring whether it reads
    /// images. `None` is the scripted judge of the scene.
    pub bridge: Option<bool>,
    /// How long each call of the judge takes, call by call. Later calls
    /// take `think`.
    pub thinks: VecDeque<Duration>,
    /// Whether a sleep of the run never ends.
    pub stall_sleep: bool,
    /// The first run of the reset command, from 1, that does not finish.
    /// The clock then lets its deadline pass at once.
    pub reset_hangs_from: Option<usize>,
    /// The trial whose browser never finishes opening, from 1.
    pub hang_launch: Option<usize>,
    /// The memory files by origin before the run.
    pub memories: BTreeMap<String, String>,
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
            api_key_env: None,
            auth: false,
            forbidden: Vec::new(),
            bridge: None,
            thinks: VecDeque::new(),
            stall_sleep: false,
            hang_launch: None,
            reset_hangs_from: None,
            memories: BTreeMap::new(),
            trial: 1,
            aliases: Aliases::default(),
        }
    }
}

impl Scene {
    /// The first allowed origin, such as `https://shop.example.test`.
    pub fn origin(&self) -> String {
        let first = found(self.origins.first(), "the background names an origin");
        first.trim_end_matches('/').to_owned()
    }

    /// Hides e-mail addresses.
    pub fn hide_emails(&mut self) {
        self.redact.push(Rule::pattern(EMAIL_RULE, EMAIL));
    }

    /// Resets with the script that acts by its run: run n moves the clock
    /// to the n-th of `ends`, and the runs from `hangs_from` on do not
    /// finish within `timeout_secs`. A script that does not finish needs
    /// `hangs_from`, or the clock waits for it.
    pub fn reset_by_run(
        &mut self,
        ends: &[Duration],
        hangs_from: Option<usize>,
        timeout_secs: u64,
    ) {
        self.reset_with("runs.sh", timeout_secs);
        let ends: Vec<String> = ends.iter().map(|end| millis(*end).to_string()).collect();
        self.env
            .push(("RESET_ENDS_MS".into(), ends.join(" ").into()));
        let from = hangs_from.map(|from| ("RESET_HANG_FROM".into(), from.to_string().into()));
        self.env.extend(from);
        self.reset_hangs_from = hangs_from;
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
