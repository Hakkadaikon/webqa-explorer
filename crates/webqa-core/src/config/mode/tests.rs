//! Tests of the check that depends on the kind of run.

use super::*;
use crate::config::raw::parse;
use crate::config::raw::tests::example;
use crate::config::validate::validate;

/// The example with `ops` set to `ops`, validated with every environment
/// variable set and a storage state in Playwright's format.
fn with_ops(ops: &str) -> Config {
    let raw = parse(&format!("{}ops: {ops}\n", example())).unwrap();
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
