//! Tests of the check that depends on the kind of run.

use super::*;
use crate::config::raw::parse;
use crate::config::raw::tests::example;
use crate::config::validate::validate;

/// The example with `ops` set to `ops`, validated with every environment
/// variable set and a storage state in Playwright's format.
fn with_ops(ops: &str) -> Config {
    validated(&format!("{}ops: {ops}\n", example()))
}

/// `yaml` validated with every environment variable set and a storage state
/// in Playwright's format.
fn validated(yaml: &str) -> Config {
    let raw = parse(yaml).unwrap();
    let state = r#"{"cookies":[],"origins":[]}"#;
    validate(
        raw,
        &|name| Some(name.to_owned()),
        &|_| Ok(state.to_owned()),
    )
    .unwrap()
}

/// The result of the check, with the error as its message.
fn outcome(config: &Config, mode: Mode) -> Result<(), String> {
    check_mode(config, mode).map_err(|err| err.to_string())
}

/// Rejected in exploration, accepted in a scenario.
const WITHOUT_ELEMENT_OPS: [&str; 3] =
    ["[done]", "[scroll_down, back]", "[scroll_up, wait, stuck]"];

/// Accepted in both kinds of run.
const WITH_ELEMENT_OPS: [&str; 4] = [
    "[click]",
    "[input]",
    "[done, input]",
    "[scroll_up, click, wait]",
];

const BOTH: [Mode; 2] = [Mode::Explore, Mode::Scenario];

fn needs_element_op() -> Result<(), String> {
    Err("探索モードでは ops に click か input を1つ以上書く".to_owned())
}

#[test]
fn explore_mode_needs_click_or_input() {
    let want = needs_element_op();
    assert_eq!(
        Err(ConfigError::ExploreNeedsElementOp.to_string()),
        want,
        "message"
    );
    for ops in WITHOUT_ELEMENT_OPS {
        let config = with_ops(ops);
        assert_eq!(outcome(&config, Mode::Explore), want, "{ops} explore");
        assert_eq!(outcome(&config, Mode::Scenario), Ok(()), "{ops} scenario");
    }
    accepts_element_ops_in_both_modes();
}

fn accepts_element_ops_in_both_modes() {
    for ops in WITH_ELEMENT_OPS {
        let config = with_ops(ops);
        for mode in BOTH {
            assert_eq!(outcome(&config, mode), Ok(()), "{ops} {mode:?}");
        }
    }
}

#[test]
fn explore_check_with_empty_ops() {
    let mut config = with_ops("[click]");
    config.ops = Vec::new();
    assert_eq!(
        outcome(&config, Mode::Explore),
        needs_element_op(),
        "explore"
    );
    assert_eq!(outcome(&config, Mode::Scenario), Ok(()), "scenario");
}

#[test]
fn scenario_mode_needs_stages() {
    let without = validated(&example()[..example().find("scenario:").unwrap()]);
    assert_eq!(
        outcome(&without, Mode::Scenario),
        Err(ConfigError::NoStages.to_string()),
        "scenario without stages"
    );
    assert_eq!(outcome(&without, Mode::Explore), Ok(()), "explore");
    let example = validated(example());
    for mode in BOTH {
        assert_eq!(outcome(&example, mode), Ok(()), "example {mode:?}");
    }
}

#[test]
fn scenario_mode_needs_a_kind_besides_enter() {
    let want = Err("決め打ちシナリオでは ops に enter のほかの操作種別を1つ以上書く".to_owned());
    assert_eq!(
        Err(ConfigError::ScenarioNeedsOpBesidesEnter.to_string()),
        want,
        "message"
    );
    for ops in ["[enter]", "[enter, enter]"] {
        let config = with_ops(ops);
        assert_eq!(outcome(&config, Mode::Scenario), want, "{ops} scenario");
        assert_eq!(
            outcome(&config, Mode::Explore),
            needs_element_op(),
            "{ops} explore"
        );
    }
    for ops in ["[enter, click]", "[enter, back]"] {
        assert_eq!(outcome(&with_ops(ops), Mode::Scenario), Ok(()), "{ops}");
    }
}
