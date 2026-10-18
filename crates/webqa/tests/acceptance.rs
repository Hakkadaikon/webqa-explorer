//! The acceptance scenarios, run with fake ports.
//!
//! Only the scenarios with steps run; the others come with later issues.

#[path = "steps/background.rs"]
mod background;
#[path = "steps/boundaries.rs"]
mod boundaries;
#[path = "support/fakes.rs"]
mod fakes;
#[path = "support/world.rs"]
mod world;

use cucumber::World as _;

/// The scenarios that have their steps.
const READY: [&str; 1] = ["判断モデルを差し替えても同じ探索ができる"];

/// The copy of the feature, read at build time to check the names in [`READY`].
const FEATURE: &str = include_str!("features/web_exploratory_qa.feature");

/// How many scenarios of the feature are called `name`.
fn scenarios_named(name: &str) -> usize {
    let heading = format!("Scenario: {name}");
    FEATURE
        .lines()
        .filter(|line| line.trim() == heading)
        .count()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    for name in READY {
        assert_eq!(scenarios_named(name), 1, "one scenario is called {name}");
    }
    let features = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/features/web_exploratory_qa.feature"
    );
    world::QaWorld::cucumber()
        .fail_on_skipped()
        .filter_run_and_exit(features, |_, _, scenario| {
            READY.contains(&scenario.name.as_str())
        })
        .await;
}
