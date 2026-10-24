//! The acceptance scenarios, run with fake ports.
//!
//! A scenario runs only when its name is in the `READY` of a steps file;
//! the others come with later issues.

#[path = "support/browser.rs"]
mod browser;
#[path = "support/clock.rs"]
mod clock;
#[path = "support/fakes.rs"]
mod fakes;
#[path = "support/interruption.rs"]
mod interruption;
#[path = "support/judge.rs"]
mod judge;
#[path = "support/page.rs"]
mod page;
#[path = "support/reason.rs"]
mod reason;
#[path = "support/remembered.rs"]
mod remembered;
#[path = "support/scene.rs"]
mod scene;
#[path = "support/setting.rs"]
mod setting;
#[path = "support/shared.rs"]
mod shared;
#[path = "support/store.rs"]
mod store;
#[path = "support/world.rs"]
mod world;

#[path = "steps/background.rs"]
mod background;
#[path = "steps/boundaries.rs"]
mod boundaries;
#[path = "steps/common.rs"]
mod common;
#[path = "steps/config.rs"]
mod config;
#[path = "steps/config_limits.rs"]
mod config_limits;
#[path = "steps/ending.rs"]
mod ending;
#[path = "steps/ending_budget.rs"]
mod ending_budget;
#[path = "steps/ending_explore.rs"]
mod ending_explore;
#[path = "steps/ending_judge.rs"]
mod ending_judge;
#[path = "steps/ending_stuck.rs"]
mod ending_stuck;
#[path = "steps/ending_time.rs"]
mod ending_time;
#[path = "steps/exit.rs"]
mod exit;
#[path = "steps/exit_scenes.rs"]
mod exit_scenes;
#[path = "steps/explore_input.rs"]
mod explore_input;
#[path = "steps/guard.rs"]
mod guard;
#[path = "steps/guard_pages.rs"]
mod guard_pages;
#[path = "steps/guard_stale.rs"]
mod guard_stale;
#[path = "steps/interrupt.rs"]
mod interrupt;
#[path = "steps/issues.rs"]
mod issues;
#[path = "steps/load.rs"]
mod load;
#[path = "steps/load_access.rs"]
mod load_access;
#[path = "steps/memory.rs"]
mod memory;
#[path = "steps/recovery.rs"]
mod recovery;
#[path = "steps/recovery_domain.rs"]
mod recovery_domain;
#[path = "steps/stages.rs"]
mod stages;
#[path = "steps/stages_given.rs"]
mod stages_given;
#[path = "steps/step.rs"]
mod step;
#[path = "steps/step_reply.rs"]
mod step_reply;
#[path = "steps/step_run.rs"]
mod step_run;

use std::collections::BTreeSet;
use std::time::Duration;

use cucumber::World as _;

/// The copy of the feature, read at build time to check the names in the
/// `READY` lists.
const FEATURE: &str = include_str!("features/web_exploratory_qa.feature");

/// The scenarios that have their steps: the `READY` of every steps file.
fn ready() -> Vec<&'static str> {
    [
        common::READY,
        boundaries::READY,
        step::READY,
        guard::READY,
        load::READY,
        ending::READY,
        ending_stuck::READY,
        ending_budget::READY,
        ending_time::READY,
        ending_explore::READY,
        ending_judge::READY,
        explore_input::READY,
        stages::READY,
        recovery::READY,
        memory::READY,
        issues::READY,
        config::READY,
        config_limits::READY,
        interrupt::READY,
        exit::READY,
        exit_scenes::READY,
    ]
    .concat()
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
/// same one.
fn check_ready(names: &[&str]) {
    for name in names {
        assert_eq!(scenarios_named(name), 1, "one scenario is called {name}");
    }
    let distinct: BTreeSet<&&str> = names.iter().collect();
    assert_eq!(distinct.len(), names.len(), "no two files claim a scenario");
}

/// The pages of a scene are the pages of `fakes::page`, with more facts.
/// The fakes of the loop's own tests are checked here too, so that this
/// test uses all of the shared file.
async fn check_fakes() {
    use webqa_core::ports::{Browser, Clock};

    let origin = "https://app.example.test";
    let shown = page::PageSpec::at("/")
        .with(page::PageSpec::button("保存"))
        .build(origin);
    let plain = fakes::page(&shown.screen.url, &[("button", "保存")]);
    assert_eq!(
        (&plain.nodes, &plain.candidates, &plain.text),
        (&shown.nodes, &shown.candidates, &shown.text),
        "the same nodes, candidates and text"
    );
    let mut browser = fakes::FakeBrowser::default();
    browser.pages.push_back(plain.clone());
    assert_eq!(
        browser.observe().await.ok(),
        Some(plain),
        "the page is shown"
    );
    let judge = fakes::RecordingJudge::default();
    assert_eq!(judge.texts.len(), 0, "nothing was asked");
    assert_eq!(
        fakes::FakeClock::default().elapsed(),
        Duration::ZERO,
        "no time"
    );
    assert_eq!(fakes::VIEWPORT.width, 100, "the viewport of the fakes");
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let names = ready();
    check_ready(&names);
    check_fakes().await;
    page::checks::check();
    scene::check();
    reason::check();
    remembered::check();
    setting::check();
    judge::checks::check().await;
    clock::check().await;
    interruption::check().await;
    browser::check().await;
    store::check().await;
    world::checks::check().await;
    let features = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/features/web_exploratory_qa.feature"
    );
    world::QaWorld::cucumber()
        .fail_on_skipped()
        .filter_run_and_exit(features, move |_, _, scenario| {
            names.contains(&scenario.name.as_str())
        })
        .await;
}
