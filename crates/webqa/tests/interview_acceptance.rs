//! The acceptance scenarios of the interview mode, run with fake ports.
//!
//! A scenario runs when its name is in the `READY` of a steps file, and
//! is left to another test when it is in `DELEGATED`; every scenario of
//! the feature is in one of the two. The scenarios of the exploring mode
//! are in `acceptance.rs`, with a world of their own.

#[path = "support/clock.rs"]
mod clock;
#[path = "support/fakes.rs"]
mod fakes;
#[path = "support/interruption.rs"]
mod interruption;
#[path = "support/page.rs"]
mod page;

#[path = "support/interview/bridge.rs"]
mod bridge;
#[path = "support/interview/checks.rs"]
mod checks;
#[path = "support/interview/judge.rs"]
mod judge;
#[path = "support/interview/launch.rs"]
mod launch;
#[path = "support/interview/ledger.rs"]
mod ledger;
#[path = "support/interview/clock.rs"]
mod run_clock;
#[path = "support/interview/scene.rs"]
mod scene;
#[path = "support/interview/start.rs"]
mod start;
#[path = "support/interview/world.rs"]
mod world;

#[path = "steps/interview/common.rs"]
mod common;
#[path = "steps/interview/endings.rs"]
mod endings;
#[path = "steps/interview/guide.rs"]
mod guide;
#[path = "steps/interview/input.rs"]
mod input;
#[path = "steps/interview/issues.rs"]
mod issues;
#[path = "steps/interview/ledger.rs"]
mod ledger_rule;
#[path = "steps/interview/modes.rs"]
mod modes;
#[path = "steps/interview/persona.rs"]
mod persona;
#[path = "steps/interview/questions.rs"]
mod questions;
#[path = "steps/interview/rerun.rs"]
mod rerun;
#[path = "steps/interview/select.rs"]
mod select;
#[path = "steps/interview/trials.rs"]
mod trials;

use std::collections::BTreeSet;

use cucumber::World as _;

/// The copy of the feature, read at build time to check the names in the
/// `READY` lists.
const FEATURE: &str = include_str!("features/interview_mode.feature");

/// The feature of the specification, which the copy must equal.
const SPECIFICATION: &str = include_str!("../../../tasks/loopeng/InterviewMode.feature");

/// The scenarios that have their steps: the `READY` of every steps file.
fn ready() -> Vec<&'static str> {
    [
        common::READY,
        input::READY,
        modes::READY,
        trials::READY,
        questions::READY,
        endings::READY,
        persona::READY,
        issues::READY,
        ledger_rule::READY,
        select::READY,
        rerun::READY,
        guide::READY,
    ]
    .concat()
}

/// How many scenarios the feature has: its Scenario and Scenario Outline
/// headings, Outline rows not counted.
fn scenarios_in_feature() -> usize {
    let headed = |line: &&str| {
        let line = line.trim_start();
        line.starts_with("Scenario:") || line.starts_with("Scenario Outline:")
    };
    FEATURE.lines().filter(headed).count()
}

/// How many scenarios of the feature are called `name`.
fn scenarios_named(name: &str) -> usize {
    let headings = [
        format!("Scenario: {name}"),
        format!("Scenario Outline: {name}"),
    ];
    FEATURE
        .lines()
        .filter(|line| headings.iter().any(|heading| line.trim() == heading))
        .count()
}

/// Every name is one scenario of the feature, and no two files claim the
/// same one. The scenarios left to another test are named too, and are not
/// run here.
fn check_ready(names: &[&str]) {
    let all = [names, modes::DELEGATED].concat();
    for name in &all {
        assert_eq!(scenarios_named(name), 1, "one scenario is called {name}");
    }
    let distinct: BTreeSet<&&str> = all.iter().collect();
    assert_eq!(distinct.len(), all.len(), "no two files claim a scenario");
    assert_eq!(
        all.len(),
        scenarios_in_feature(),
        "every scenario is run here or left to another test"
    );
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    assert_eq!(FEATURE, SPECIFICATION, "the copy is the specification");
    let names = ready();
    check_ready(&names);
    page::checks::check();
    clock::check().await;
    interruption::check().await;
    checks::check().await;
    let features = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/features/interview_mode.feature"
    );
    world::InterviewWorld::cucumber()
        .fail_on_skipped()
        .filter_run_and_exit(features, move |_, _, scenario| {
            names.contains(&scenario.name.as_str())
        })
        .await;
}
