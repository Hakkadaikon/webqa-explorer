//! The configuration as read from YAML, before validation.

use std::collections::BTreeMap;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;

use super::ConfigError;

/// The whole configuration file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawConfig {
    /// The service under test.
    pub target: RawTarget,
    /// The judge model.
    pub judge: RawJudge,
    /// Thresholds and limits.
    pub limits: Limits,
    /// Guardrails.
    pub guard: Guard,
    /// Values to hide.
    #[serde(default)]
    pub redact: Vec<RawRedact>,
    /// Values to type into inputs, keyed by the input's accessible name.
    #[serde(default)]
    pub inputs: BTreeMap<String, EnvRef>,
    /// Authentication state.
    pub auth: Option<Auth>,
    /// A scripted scenario.
    pub scenario: Option<Scenario>,
    /// Operation kinds in use. All eight when omitted.
    #[serde(default = "all_ops")]
    pub ops: Vec<Op>,
}

/// The service under test.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTarget {
    /// The start URL.
    pub url: String,
    /// Allowed origins. A host may start with `*.`.
    pub allowed_origins: Vec<String>,
}

/// The judge model.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawJudge {
    /// The adapter kind.
    pub kind: String,
    /// The bridge command.
    pub command: Option<Vec<String>>,
    /// The environment variable holding the API key.
    pub api_key: EnvRef,
    /// How long to wait for an answer.
    pub timeout_secs: NonZeroU64,
    /// Prices used to compute the cost.
    pub pricing: Pricing,
}

/// A reference to an environment variable.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvRef {
    /// The variable name.
    pub env: String,
}

/// Prices per 1000 tokens.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pricing {
    /// Price of 1000 input tokens.
    pub input_per_1k: f64,
    /// Price of 1000 output tokens.
    pub output_per_1k: f64,
}

/// Thresholds and limits.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// Unchanged screens in a row that count as frozen.
    pub freeze_threshold: NonZeroU32,
    /// Steps without progress that count as stuck.
    pub stuck_threshold: NonZeroU32,
    /// The most judge calls in a run.
    pub max_decisions: NonZeroU32,
    /// The most operations in an hour.
    pub requests_per_hour: NonZeroU32,
    /// How long to wait for a page.
    pub page_timeout_secs: NonZeroU64,
    /// How long a run may last. No time limit when omitted.
    pub max_duration_secs: Option<NonZeroU64>,
    /// Confidence needed to go back.
    pub back_confidence: f64,
    /// Confidence needed to record an issue.
    pub error_confidence: f64,
}

/// Guardrails.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guard {
    /// What must not be touched.
    pub forbidden: Forbidden,
    /// What may be touched despite `forbidden`.
    pub allowed: Allowed,
    /// What to do on a paywall. `back` when omitted.
    #[serde(default = "default_leave_paywall")]
    pub leave_paywall: String,
}

/// What must not be touched.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Forbidden {
    /// Keywords.
    pub keywords: Vec<String>,
    /// CSS selectors.
    pub selectors: Vec<String>,
}

/// What may be touched despite `forbidden`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Allowed {
    /// CSS selectors.
    pub selectors: Vec<String>,
}

/// A value to hide.
#[derive(Debug, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum RawRedact {
    /// A value matched by a regular expression.
    Pattern {
        /// The name written in place of the value.
        name: String,
        /// The regular expression.
        pattern: String,
    },
    /// A value read from an environment variable.
    Env {
        /// The name written in place of the value.
        name: String,
        /// The variable name.
        env: String,
    },
}

/// Authentication state.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Auth {
    /// Path to a Playwright storage state file.
    pub storage_state: Option<PathBuf>,
}

/// A scripted scenario.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    /// Stages in order.
    pub stages: Vec<Stage>,
}

/// One stage of a scenario.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    /// The stage name.
    pub name: String,
    /// The goal given to the judge.
    pub goal: String,
    /// When the stage is done.
    pub done_when: DoneWhen,
}

/// When a stage is done.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DoneWhen {
    /// The URL path reached.
    pub url: Option<String>,
    /// An element shown.
    pub element: Option<Element>,
    /// A text shown.
    pub text: Option<String>,
}

/// An element by role and accessible name.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Element {
    /// The ARIA role.
    pub role: String,
    /// The accessible name.
    pub name: String,
}

/// An operation kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    /// Click an element.
    Click,
    /// Type into an element.
    Input,
    /// Scroll down.
    ScrollDown,
    /// Scroll up.
    ScrollUp,
    /// Go back.
    Back,
    /// Wait.
    Wait,
    /// Declare the stage done.
    Done,
    /// Declare being stuck.
    Stuck,
}

fn all_ops() -> Vec<Op> {
    vec![
        Op::Click,
        Op::Input,
        Op::ScrollDown,
        Op::ScrollUp,
        Op::Back,
        Op::Wait,
        Op::Done,
        Op::Stuck,
    ]
}

fn default_leave_paywall() -> String {
    "back".to_owned()
}

/// Reads the YAML into the raw types. Unknown fields are rejected.
///
/// # Errors
///
/// [`ConfigError::LiteralApiKey`] when `judge.api_key` is not
/// `{ env: <name> }`, and [`ConfigError::Yaml`] for any other problem.
/// Neither shows the YAML itself.
pub fn parse(yaml: &str) -> Result<RawConfig, ConfigError> {
    reject_literal_key(yaml)?;
    Ok(serde_saphyr::from_str_with_options(yaml, options())?)
}

fn options() -> serde_saphyr::Options {
    serde_saphyr::options! { with_snippet: false }
}

/// Only `judge.api_key`, read leniently so that a key written in place never
/// reaches a type mismatch error, which would quote it.
#[derive(Deserialize)]
struct Probe {
    judge: ProbeJudge,
}

#[derive(Deserialize)]
struct ProbeJudge {
    api_key: Value,
}

fn reject_literal_key(yaml: &str) -> Result<(), ConfigError> {
    let probe = serde_saphyr::from_str_with_options::<Probe>(yaml, options());
    if probe.is_ok_and(|p| !p.judge.api_key.is_object()) {
        Err(ConfigError::LiteralApiKey)
    } else {
        Ok(())
    }
}

/// Shared by the tests of the configuration.
#[cfg(test)]
pub(crate) mod tests {
    use std::mem::discriminant;

    use super::*;

    /// The example from `docs/architecture.md`, without comments.
    pub const EXAMPLE: &str = r##"target:
  url: https://app.example.test
  allowed_origins: ["https://app.example.test", "https://*.app.example.test"]
judge:
  kind: command
  command: ["./bridge/jev.py"]
  api_key: { env: JEV_API_KEY }
  timeout_secs: 30
  pricing: { input_per_1k: 0.0, output_per_1k: 0.0 }
limits:
  freeze_threshold: 3
  stuck_threshold: 3
  max_decisions: 200
  requests_per_hour: 600
  page_timeout_secs: 10
  max_duration_secs: 60
  back_confidence: 0.7
  error_confidence: 0.8
guard:
  forbidden: { keywords: ["購入"], selectors: ["#danger"] }
  allowed: { selectors: [] }
  leave_paywall: back
redact:
  - { name: メールアドレス, pattern: "[\\w.+-]+@[\\w-]+\\.[\\w.]+" }
inputs:
  メールアドレス: { env: TEST_EMAIL }
auth:
  storage_state: ./auth.json
scenario:
  stages:
    - name: ログイン
      goal: ログインを済ませる
      done_when:
        url: /home
        element: { role: button, name: ログアウト }
"##;

    /// `EXAMPLE` with `from` replaced by `to`. Fails if `from` is absent.
    pub fn changed(from: &str, to: &str) -> String {
        let yaml = EXAMPLE.replacen(from, to, 1);
        assert_ne!(yaml, EXAMPLE, "the example has no {from:?}");
        yaml
    }

    #[test]
    fn reads_target_inputs_and_stages() {
        let raw = parse(EXAMPLE).unwrap();
        assert_eq!(raw.target.url, "https://app.example.test", "target url");
        let email = raw.inputs.get("メールアドレス").unwrap();
        assert_eq!(email.env, "TEST_EMAIL", "input env");
        let stages = raw.scenario.unwrap().stages;
        assert_eq!(stages.len(), 1, "one stage");
        let login = &stages[0];
        assert_eq!(login.name, "ログイン", "stage name");
        assert_eq!(
            login.done_when.url.as_deref(),
            Some("/home"),
            "done_when url"
        );
        let element = login.done_when.element.as_ref().unwrap();
        assert_eq!(element.role, "button", "element role");
        assert_eq!(element.name, "ログアウト", "element name");
        reads_two_stages_in_order();
    }

    fn reads_two_stages_in_order() {
        let two = format!(
            "{}    - name: 注文\n      goal: 注文を済ませる\n      done_when:\n        text: 注文完了\n",
            changed("  leave_paywall: back\n", "")
        );
        let raw = parse(&two).unwrap();
        let stages = raw.scenario.unwrap().stages;
        let names: Vec<_> = stages.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["ログイン", "注文"], "stages in order");
        let text = stages[1].done_when.text.as_deref();
        assert_eq!(text, Some("注文完了"), "second done_when text");
        assert_eq!(raw.guard.leave_paywall, "back", "leave_paywall default");
    }

    #[test]
    fn rejects_tls_verification_key() {
        let judge = changed(
            "  timeout_secs: 30\n",
            "  timeout_secs: 30\n  tls_verify: false\n",
        );
        let err = parse(&judge).unwrap_err();
        assert!(
            err.to_string().contains("tls_verify"),
            "unknown field: {err}"
        );
        let target = changed(
            "  url: https://app.example.test\n",
            "  url: https://app.example.test\n  insecure: true\n",
        );
        let err = parse(&target).unwrap_err();
        assert!(err.to_string().contains("insecure"), "unknown field: {err}");
    }

    #[test]
    fn literal_api_key_rejected_without_value() {
        let yaml = changed(
            "api_key: { env: JEV_API_KEY }",
            "api_key: \"sk-test-123456\"",
        );
        let err = parse(&yaml).unwrap_err();
        assert_eq!(
            discriminant(&err),
            discriminant(&ConfigError::LiteralApiKey),
            "{err:?}"
        );
        assert_hides_key(&err);
        // Broken YAML next to the key: the error must not quote the lines.
        let broken = yaml.replacen("  timeout_secs: 30\n", "  timeout_secs: [\n", 1);
        assert_ne!(broken, yaml, "the YAML has timeout_secs");
        assert_hides_key(&parse(&broken).unwrap_err());
    }

    fn assert_hides_key(err: &ConfigError) {
        assert!(
            !err.to_string().contains("sk-test-123456"),
            "Display hides the key: {err}"
        );
        assert!(
            !format!("{err:?}").contains("sk-test-123456"),
            "Debug hides the key"
        );
    }
}
