//! Loading and validating the YAML configuration.
//!
//! Reading is split into two steps: [`raw::parse`] reads the YAML into raw
//! types, and [`validate::validate`] turns them into a checked [`Config`].

pub mod interview;
mod mode;
pub mod persona;
pub mod purpose;
pub mod raw;
pub mod validate;

pub use mode::check_mode;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::num::NonZeroU64;

use url::Url;

use raw::{Auth, Guard, Limits, Op, Pricing, Scenario};

/// Why the configuration was rejected. No variant carries a secret value.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The YAML could not be read into the raw types.
    #[error("設定の YAML を読めない: {0}")]
    Yaml(#[from] serde_saphyr::Error),
    /// `judge.api_key` is not exactly `{ env: <name> }`, so it may hold the
    /// key itself.
    #[error("API キーを設定ファイルに直接書かず、`{{ env: 変数名 }}` で環境変数を指定する")]
    LiteralApiKey,
    /// A referenced environment variable is not set.
    #[error("環境変数 {0} が設定されていない")]
    MissingEnv(String),
    /// A confidence threshold is not a finite value in `(0, 1]`.
    #[error("{0} は 0 より大きく 1 以下の値にする")]
    Confidence(&'static str),
    /// A price is negative or not a number.
    #[error("{0} は 0 以上の値にする")]
    Pricing(&'static str),
    /// `scenario.stages` is empty.
    #[error("scenario.stages に段階を1つ以上書く")]
    NoStages,
    /// The list of operation kinds is empty.
    #[error("ops に操作種別を1つ以上書く")]
    NoOps,
    /// A field could not be read as a URL.
    #[error("{0} を URL として読めない")]
    InvalidUrl(&'static str),
    /// The storage state file is unreadable or not in Playwright's format.
    #[error("auth.storage_state を Playwright の保存形式として読めない")]
    StorageState,
    /// Exploring with no operation kind that acts on an element.
    #[error("探索モードでは ops に click か input を1つ以上書く")]
    ExploreNeedsElementOp,
    /// A scripted scenario whose only operation kind is `enter`.
    #[error("決め打ちシナリオでは ops に enter のほかの操作種別を1つ以上書く")]
    ScenarioNeedsOpBesidesEnter,
    /// `guard.leave_paywall` names no operation that leaves without a target.
    #[error(
        "guard.leave_paywall は back、scroll_down、scroll_up、wait、reload、return のどれかにする"
    )]
    LeavePaywall,
    /// A problem with the input of an interview run.
    #[error(transparent)]
    Interview(#[from] interview::InterviewError),
}

/// A secret value. Neither `Debug` nor any other formatting shows it.
pub struct Secret(String);

impl Secret {
    /// The secret value itself.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

/// The checked configuration.
#[derive(Debug)]
pub struct Config {
    /// The service under test.
    pub target: Target,
    /// The judge model.
    pub judge: Judge,
    /// Thresholds and limits.
    pub limits: Limits,
    /// Guardrails.
    pub guard: Guard,
    /// Values to hide before they leave the process.
    pub redact: Vec<Redact>,
    /// Values to type into inputs, keyed by the input's accessible name.
    pub inputs: BTreeMap<String, Secret>,
    /// The keys of `inputs` written with `secret: false`. Their values are
    /// not hidden from the judge.
    pub open_inputs: BTreeSet<String>,
    /// Authentication state. A storage state file is already checked for
    /// Playwright's format.
    pub auth: Option<Auth>,
    /// Stages of a scripted scenario. `None` means exploration only.
    pub scenario: Option<Scenario>,
    /// Operation kinds in use. Never empty.
    pub ops: Vec<Op>,
    /// The interview settings as written, checked only for an interview run.
    pub interview: Option<interview::RawInterview>,
}

/// The service under test.
#[derive(Debug)]
pub struct Target {
    /// The start URL.
    pub url: Url,
    /// Allowed origins as written. Each one parses as a URL.
    pub allowed_origins: Vec<String>,
}

/// The judge model.
#[derive(Debug)]
pub struct Judge {
    /// The adapter kind.
    pub kind: String,
    /// The bridge command, if the adapter runs one.
    pub command: Option<Vec<String>>,
    /// The API key read from the environment.
    ///
    /// `None` when the configuration names none: authentication is then left
    /// outside (the Network secrets relay or the bridge command). The bridge
    /// sends no authentication header, and the proxy adds the key.
    pub api_key: Option<Secret>,
    /// How long to wait for an answer.
    pub timeout_secs: NonZeroU64,
    /// Prices used to compute the cost.
    pub pricing: Pricing,
}

/// A value to hide, with the name written in its place.
#[derive(Debug)]
pub struct Redact {
    /// The name written in place of the value.
    pub name: String,
    /// Where the value comes from.
    pub source: RedactSource,
}

/// Where a value to hide comes from.
#[derive(Debug)]
pub enum RedactSource {
    /// A regular expression matching the value.
    Pattern(String),
    /// The value read from an environment variable.
    Env(Secret),
}
