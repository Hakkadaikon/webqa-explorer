//! Rules made from the checked configuration.

use std::path::Path;
use std::time::Duration;

use super::*;
use crate::config::raw::parse;
use crate::config::raw::tests::{changed, example};
use crate::config::validate::validate;
use crate::machine::testing::{ALL_OPS, nz};

const ORIGINS: &str =
    "  allowed_origins: [\"https://app.example.test\", \"https://*.app.example.test\"]\n";

fn config(yaml: &str) -> Config {
    let env = |name: &str| Some(format!("value-of-{name}"));
    let read = |_: &Path| Ok(r#"{"cookies":[],"origins":[]}"#.to_owned());
    validate(parse(yaml).unwrap(), &env, &read).unwrap()
}

fn origins(values: &[&str]) -> Vec<AllowedOrigin> {
    values
        .iter()
        .map(|v| AllowedOrigin::parse(v).unwrap())
        .collect()
}

#[test]
fn rules_from_config() {
    let yaml = changed("[\"購入\"]", "[\"公開する\"]");
    let rules = Rules::from_config(&config(&yaml), Mode::Scenario).unwrap();
    let expected = Rules {
        mode: Mode::Scenario,
        ops: ALL_OPS.to_vec(),
        freeze_threshold: nz(3),
        stuck_threshold: nz(3),
        max_decisions: nz(200),
        requests_per_hour: nz(600),
        max_duration: Some(Duration::from_secs(60)),
        back_confidence: 0.7,
        allowed: origins(&["https://app.example.test", "https://*.app.example.test"]),
        forbidden: Forbidden::new(&["公開する".to_owned()]),
        paywall: Paywall::default(),
        leave_paywall: "back".to_owned(),
    };
    assert_eq!(rules, expected, "the values of the configuration");
}

#[test]
fn empty_origins_allow_the_start_url() {
    let yaml = changed(ORIGINS, "  allowed_origins: []\n");
    let rules = Rules::from_config(&config(&yaml), Mode::Explore).unwrap();
    assert_eq!(
        rules.allowed,
        origins(&["https://app.example.test"]),
        "the start"
    );
    assert_eq!(rules.mode, Mode::Explore, "the mode passed");
}

#[test]
fn other_values_follow_the_configuration() {
    let yaml = changed(
        "  max_duration_secs: 60    # 書かなければ時間切れは起きない\n",
        "",
    );
    let yaml = yaml.replacen("  leave_paywall: back\n", "  leave_paywall: close\n", 1);
    let yaml = format!("{yaml}ops: [click, back]\n");
    let rules = Rules::from_config(&config(&yaml), Mode::Scenario).unwrap();
    assert_eq!(rules.max_duration, None, "no time limit");
    assert_eq!(rules.leave_paywall, "close", "the way out");
    assert_eq!(rules.ops, [Op::Click, Op::Back], "the operation kinds");
}

#[test]
fn invalid_origin_is_rejected() {
    let mut config = config(example());
    let path = "https://app.example.test/path".to_owned();
    config.target.allowed_origins.push(path);
    let rules = Rules::from_config(&config, Mode::Scenario);
    assert_eq!(rules, Err(InvalidOrigin), "an origin with a path");
}
