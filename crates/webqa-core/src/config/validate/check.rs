//! Checks that only reject a value, without reading secrets.

use url::Url;

use super::super::ConfigError;
use super::super::raw::{Op, RawConfig, Scenario};
use super::Read;
use super::storage_state::storage_state;

/// All the checks, in the order the errors are reported.
pub fn check(raw: &RawConfig, read: Read<'_>) -> Result<(), ConfigError> {
    let pricing = &raw.judge.pricing;
    [
        price("judge.pricing.input_per_1k", pricing.input_per_1k),
        price("judge.pricing.output_per_1k", pricing.output_per_1k),
        confidence("limits.back_confidence", raw.limits.back_confidence),
        confidence("limits.error_confidence", raw.limits.error_confidence),
        non_empty_ops(&raw.ops),
        non_empty_stages(raw.scenario.as_ref()),
        origins(&raw.target.allowed_origins),
        storage_state(raw.auth.as_ref(), read),
    ]
    .into_iter()
    .collect()
}

/// Not negative and not NaN.
const fn price(field: &'static str, value: f64) -> Result<(), ConfigError> {
    if value >= 0.0 {
        Ok(())
    } else {
        Err(ConfigError::Pricing(field))
    }
}

const fn confidence(field: &'static str, value: f64) -> Result<(), ConfigError> {
    if value > 0.0 && value <= 1.0 {
        Ok(())
    } else {
        Err(ConfigError::Confidence(field))
    }
}

const fn non_empty_ops(ops: &[Op]) -> Result<(), ConfigError> {
    if ops.is_empty() {
        Err(ConfigError::NoOps)
    } else {
        Ok(())
    }
}

fn non_empty_stages(scenario: Option<&Scenario>) -> Result<(), ConfigError> {
    if scenario.is_some_and(|s| s.stages.is_empty()) {
        Err(ConfigError::NoStages)
    } else {
        Ok(())
    }
}

/// Only whether each origin parses. Matching is the guardrail's job.
fn origins(origins: &[String]) -> Result<(), ConfigError> {
    origins.iter().map(String::as_str).try_for_each(origin)
}

fn origin(origin: &str) -> Result<(), ConfigError> {
    Url::parse(origin)
        .ok()
        .filter(wildcard_at_start)
        .map(drop)
        .ok_or(ConfigError::InvalidUrl("target.allowed_origins"))
}

/// A host with `*` only as a leading `*.`.
fn wildcard_at_start(url: &Url) -> bool {
    let host = url.host_str().unwrap_or_default();
    let rest = host.strip_prefix("*.").unwrap_or(host);
    !rest.is_empty() && !rest.contains('*')
}

#[cfg(test)]
mod tests {
    use std::mem::discriminant;

    use super::super::tests::load;
    use super::*;
    use crate::config::raw::parse;
    use crate::config::raw::tests::{changed, example};

    /// The error from loading the example with `from` replaced by `to`.
    fn rejected(from: &str, to: &str) -> ConfigError {
        load(&changed(from, to)).unwrap_err()
    }

    fn yaml_error() -> ConfigError {
        parse("[").unwrap_err()
    }

    /// Rejected while reading the YAML.
    const NON_POSITIVE: [(&str, &str); 8] = [
        ("freeze_threshold: 3", "freeze_threshold: 0"),
        ("stuck_threshold: 3", "stuck_threshold: 0"),
        ("max_decisions: 200", "max_decisions: 0"),
        ("requests_per_hour: 600", "requests_per_hour: 0"),
        ("max_decisions: 200", "max_decisions: -1"),
        ("freeze_threshold: 3", "freeze_threshold: -1"),
        ("page_timeout_secs: 10", "page_timeout_secs: 0"),
        ("timeout_secs: 30", "timeout_secs: 0"),
    ];

    const STAGES: &str = "  stages:\n    - name: ログイン\n      goal: ログインを済ませる\n      done_when:\n        url: /home\n        element: { role: button, name: ログアウト }\n";

    const BACK: &str = "limits.back_confidence";
    const ERROR: &str = "limits.error_confidence";
    const INPUT: &str = "judge.pricing.input_per_1k";
    const OUTPUT: &str = "judge.pricing.output_per_1k";

    /// Rejected while validating, with the error expected.
    fn invalid_values() -> [(&'static str, &'static str, ConfigError); 9] {
        [
            ("scenario:", "ops: []\nscenario:", ConfigError::NoOps),
            (
                "back_confidence: 0.7",
                "back_confidence: 0",
                ConfigError::Confidence(BACK),
            ),
            (
                "back_confidence: 0.7",
                "back_confidence: 1.5",
                ConfigError::Confidence(BACK),
            ),
            (
                "back_confidence: 0.7",
                "back_confidence: .nan",
                ConfigError::Confidence(BACK),
            ),
            (
                "error_confidence: 0.8",
                "error_confidence: 0",
                ConfigError::Confidence(ERROR),
            ),
            (
                "input_per_1k: 0.0",
                "input_per_1k: -1.0",
                ConfigError::Pricing(INPUT),
            ),
            (
                "output_per_1k: 0.0",
                "output_per_1k: .nan",
                ConfigError::Pricing(OUTPUT),
            ),
            (STAGES, "  stages: []\n", ConfigError::NoStages),
            (
                "url: https://app.example.test",
                "url: not a url",
                ConfigError::InvalidUrl("target.url"),
            ),
        ]
    }

    /// Origins that are not URLs, or put `*` anywhere but the host's start.
    const BAD_ORIGINS: [&str; 4] = [
        "not an origin",
        "https://a.*.test",
        "https://*",
        "https://*.",
    ];

    #[test]
    fn rejects_non_positive_and_empty() {
        assert!(load(example()).is_ok(), "the example is valid");
        for (from, to) in NON_POSITIVE {
            let err = rejected(from, to);
            assert_eq!(
                discriminant(&err),
                discriminant(&yaml_error()),
                "{to}: {err}"
            );
        }
        for (from, to, want) in invalid_values() {
            assert_eq!(rejected(from, to).to_string(), want.to_string(), "{to}");
        }
        rejects_bad_origins();
        let one = changed("back_confidence: 0.7", "back_confidence: 1");
        assert!(load(&one).is_ok(), "back_confidence: 1 is accepted");
    }

    fn rejects_bad_origins() {
        let want = ConfigError::InvalidUrl("target.allowed_origins").to_string();
        for origin in BAD_ORIGINS {
            let to = format!("\"{origin}\"");
            let err = rejected("\"https://*.app.example.test\"", &to);
            assert_eq!(err.to_string(), want, "{origin}");
        }
        let wildcard = changed("[\"https://app.example.test\", ", "[");
        assert!(
            load(&wildcard).is_ok(),
            "a wildcard at the host's start is accepted"
        );
    }
}
