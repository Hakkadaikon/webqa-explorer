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

#[tokio::main(flavor = "current_thread")]
async fn main() {
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
