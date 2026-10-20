//! Tests of reading the configuration into what a run needs.

use std::io;
use std::mem::{Discriminant, discriminant};
use std::path::PathBuf;
use std::time::Duration;

use webqa_core::config::raw::{Op, parse};
use webqa_core::config::validate::validate;

use super::*;

/// The YAML example of `docs/arch/config.md`, copied as written there.
pub(super) const EXAMPLE: &str = r##"target:
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
  max_duration_secs: 60    # 書かなければ時間切れは起きない
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

/// A storage state in Playwright's format.
const STATE: &str = r#"{"cookies":[],"origins":[]}"#;

fn value_of(name: &str) -> String {
    format!("value-of-{name}")
}

/// Every environment variable is set.
pub(super) const ALL_ENV: Env<'static> = &|name| Some(value_of(name));
/// Every file holds [`STATE`].
const STATE_OK: Read<'static> = &|_| Ok(STATE.to_owned());
/// The API key is not set.
const NO_KEY: Env<'static> = &|name| (name != "JEV_API_KEY").then(|| value_of(name));
/// The input value is empty.
pub(super) const EMPTY_EMAIL: Env<'static> = &|name| {
    Some(if name == "TEST_EMAIL" {
        String::new()
    } else {
        value_of(name)
    })
};
/// No file can be read.
const UNREADABLE: Read<'static> = &|_| Err(io::Error::other("unreadable"));

/// The example with `from` replaced by `to`. Fails if `from` is absent.
pub(super) fn changed(from: &str, to: &str) -> String {
    let yaml = EXAMPLE.replacen(from, to, 1);
    assert_ne!(yaml, EXAMPLE, "the example has no {from:?}");
    yaml
}

pub(super) fn example(mode: Mode) -> Setup {
    load(EXAMPLE, mode, ALL_ENV, STATE_OK).unwrap()
}

pub(super) fn config(yaml: &str, env: Env<'_>) -> Config {
    validate(parse(yaml).unwrap(), env, STATE_OK).unwrap()
}

#[test]
fn example_config_sets_up_both_modes() {
    let setup = example(Mode::Scenario);
    assert_eq!(setup.mode, Mode::Scenario, "mode");
    let stages: Vec<&str> = setup.rules.stages.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(stages, ["ログイン"], "stages");
    let max = Some(Duration::from_secs(60));
    assert_eq!(setup.rules.max_duration, max, "max duration");
    assert_eq!(setup.origin, "https://app.example.test", "origin");
    let target = setup.target.as_inner();
    assert_eq!(target, "https://app.example.test/", "target");
    assert_eq!(setup.inputs, ["メールアドレス"], "input keys");
    let pricing = &setup.judge.pricing;
    let numbers = (
        setup.error_confidence,
        pricing.input_per_1k,
        pricing.output_per_1k,
    );
    assert_eq!(numbers, (0.8, 0.0, 0.0), "confidence and prices");
    assert_eq!(setup.judge.kind, "command", "judge kind");
    sets_up_browser(&setup.browser);
    let explore = example(Mode::Explore);
    assert_eq!(explore.rules.mode, Mode::Explore, "explore mode");
}

fn sets_up_browser(browser: &BrowserPlan) {
    assert_eq!(
        browser.page_timeout,
        Duration::from_secs(10),
        "page timeout"
    );
    let state = Some(PathBuf::from("./auth.json"));
    assert_eq!(browser.storage_state, state, "storage state");
    assert_eq!(browser.forbidden_selectors, ["#danger"], "forbidden");
    let inputs: Vec<(&str, &str)> = browser
        .inputs
        .iter()
        .map(|(key, value)| (key.as_str(), value.expose()))
        .collect();
    let want = [("メールアドレス", "value-of-TEST_EMAIL")];
    assert_eq!(inputs, want, "inputs");
}

/// A configuration that cannot start a run, with the error expected.
struct Case {
    yaml: String,
    mode: Mode,
    env: Env<'static>,
    read: Read<'static>,
    kind: Option<Discriminant<ConfigError>>,
    message: String,
}

/// The kind of configuration error, or `None` for the other errors.
fn kind(error: &SetupError) -> Option<Discriminant<ConfigError>> {
    match error {
        SetupError::Config(inner) => Some(discriminant(inner)),
        SetupError::Origin(_) | SetupError::Redact(_) => None,
    }
}

fn config_case(yaml: String, mode: Mode, env: Env<'static>, error: &ConfigError) -> Case {
    Case {
        yaml,
        mode,
        env,
        read: STATE_OK,
        kind: Some(discriminant(error)),
        message: error.to_string(),
    }
}

fn ops_done() -> String {
    changed("scenario:", "ops: [done]\nscenario:")
}

fn cases() -> Vec<Case> {
    let literal_key = changed("api_key: { env: JEV_API_KEY }", "api_key: sk-live-123");
    let frozen = changed("freeze_threshold: 3", "freeze_threshold: 0");
    let yaml_error = parse(&frozen).unwrap_err();
    let no_scenario = EXAMPLE[..EXAMPLE.find("scenario:").unwrap()].to_owned();
    let missing = ConfigError::MissingEnv("JEV_API_KEY".to_owned());
    let explore_ops = &ConfigError::ExploreNeedsElementOp;
    vec![
        config_case(
            literal_key,
            Mode::Explore,
            ALL_ENV,
            &ConfigError::LiteralApiKey,
        ),
        config_case(EXAMPLE.to_owned(), Mode::Explore, NO_KEY, &missing),
        config_case(frozen, Mode::Explore, ALL_ENV, &yaml_error),
        config_case(no_scenario, Mode::Scenario, ALL_ENV, &ConfigError::NoStages),
        config_case(ops_done(), Mode::Explore, ALL_ENV, explore_ops),
        config_case(
            changed("scenario:", "ops: [enter]\nscenario:"),
            Mode::Scenario,
            ALL_ENV,
            &ConfigError::ScenarioNeedsOpBesidesEnter,
        ),
        bad_pattern(),
        Case {
            read: UNREADABLE,
            ..config_case(
                EXAMPLE.to_owned(),
                Mode::Explore,
                ALL_ENV,
                &ConfigError::StorageState,
            )
        },
    ]
}

fn bad_pattern() -> Case {
    let pattern = r#"pattern: "[\\w.+-]+@[\\w-]+\\.[\\w.]+""#;
    Case {
        yaml: changed(pattern, r#"pattern: "(""#),
        mode: Mode::Explore,
        env: ALL_ENV,
        read: STATE_OK,
        kind: None,
        message: r#"redaction rule "メールアドレス" is not a valid regular expression"#.to_owned(),
    }
}

#[test]
fn setup_errors_carry_no_value() {
    for case in cases() {
        let error = load(&case.yaml, case.mode, case.env, case.read).unwrap_err();
        let text = error.to_string();
        let got = (kind(&error), text.as_str());
        assert_eq!(got, (case.kind, case.message.as_str()), "{}", case.yaml);
        assert!(!text.contains("value-of-"), "no value in {text}");
        assert!(!text.contains("sk-live-123"), "no key in {text}");
    }
    let scenario = load(&ops_done(), Mode::Scenario, ALL_ENV, STATE_OK).unwrap();
    assert_eq!(scenario.rules.ops, [Op::Done], "only done in a scenario");
}

#[test]
fn setup_debug_hides_values() {
    let setup = example(Mode::Scenario);
    let shown = format!("{setup:?}");
    assert!(!shown.contains("value-of-"), "no value in {shown}");
    assert!(
        shown.contains("https://app.example.test"),
        "origin in {shown}"
    );
}

const PERSONA: &str = "name: 佐藤さん
profile: 60代
it_literacy: 基本操作だけ
wants: 注文する
avoids: [URL を手で打たない]
";

/// Every file is a persona document or a storage state, by its name.
const FILES: Read<'static> = &|path| {
    Ok(if path.ends_with("persona.yaml") {
        PERSONA.to_owned()
    } else {
        STATE.to_owned()
    })
};

#[test]
fn interview_settings_are_read_with_the_setup() {
    let yaml = format!("{EXAMPLE}interview:\n  persona: persona.yaml\n  trials: 2\n");
    let (setup, interview) = load_interview(&yaml, ALL_ENV, FILES).unwrap();
    assert_eq!(setup.origin, "https://app.example.test");
    assert_eq!(interview.persona.name, "佐藤さん");
    assert_eq!(interview.trials.get(), 2);
    let missing = load_interview(EXAMPLE, ALL_ENV, FILES).err();
    assert_eq!(
        missing.map(|error| error.to_string()),
        Some("インタビューのモードでは設定に interview を書く".to_owned())
    );
}
