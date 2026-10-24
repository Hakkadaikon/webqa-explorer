//! Checks of the world: the run, and what the accessors read from it.

use std::collections::BTreeSet;
use std::time::Duration;

use webqa::exit;
use webqa::run::RunError;
use webqa::run::session::Ending;
use webqa_core::machine::{ExitReason, Step};
use webqa_core::ports::{BrowserError, StoreError};

use super::run::Fakes;
use super::{Beat, QaWorld, beat_of};
use crate::judge::Reply;

/// A world whose scene has the background and one click on "保存".
fn clicking() -> QaWorld {
    let mut world = QaWorld::default();
    world
        .scene
        .origins
        .push("https://app.example.test".to_owned());
    world.scene.limits.extend([
        ("freeze_threshold", 3),
        ("stuck_threshold", 3),
        ("max_decisions", 20),
        ("requests_per_hour", 10),
    ]);
    world.scene.show_button("保存");
    world
        .scene
        .replies
        .push_back(Reply::op("click", 1.0).on("保存"));
    world
}

/// Checks a run the harness stopped: what the accessors say about it.
async fn check_stopped() {
    let mut world = clicking();
    world.run().await;
    world.run().await;
    let reason = (world.reason(), world.asked(), world.answered());
    assert_eq!(reason, (None, 2, 1), "the second question was left waiting");
    assert_eq!(
        world.touched(),
        [("click".to_owned(), Some("保存".to_owned()))],
        "one click"
    );
    assert_eq!(
        world.explored(),
        BTreeSet::from(["保存".to_owned()]),
        "the memory has it"
    );
    assert_eq!(
        (world.when_beat(), world.when_asked()),
        (Beat::Ask, true),
        "the When asked"
    );
    check_after_stop(world);
}

/// What a run the harness stopped says after the run.
fn check_after_stop(mut world: QaWorld) {
    assert_eq!(world.window_used(), 1, "one access");
    assert_eq!(world.sleeps().len(), 0, "no sleep");
    assert!(world.report().is_some(), "the report is written");
    assert_eq!(
        world.kinds().first().map(String::as_str),
        Some("run_started"),
        "the log starts"
    );
    assert_eq!(
        world.lines().len(),
        world.kinds().len(),
        "a kind for each line"
    );
    world.poke(Duration::from_secs(60));
    assert_eq!(
        world.poked.as_ref().and_then(Step::reason),
        Some(ExitReason::UserAborted),
        "it ended"
    );
    assert_eq!(
        world.ran().interrupt.fired,
        1,
        "the gate fired the interrupt once"
    );
    assert_eq!(world.scene.marks.len(), 0, "no Rule passed a mark");
    world.scene.user_interrupts = true;
    assert_eq!(
        world.reason(),
        Some(ExitReason::UserAborted),
        "the user's interrupt"
    );
}

/// Checks the runs that end otherwise: configuration, and no outcome.
async fn check_configured() {
    let mut world = clicking();
    world.configured();
    world.run().await;
    assert_eq!(
        world.ran().config_error,
        None,
        "the base configuration is accepted"
    );
    let mut missing = clicking();
    missing.configured();
    missing.scene.env.clear();
    missing.run().await;
    assert_eq!(
        missing.ran().code,
        exit::CONFIG_ERROR,
        "the exit code of a configuration error"
    );
    assert_eq!(
        (missing.reason(), missing.ran().ending.is_none()),
        (None, true),
        "nothing ran"
    );
    for ending in [
        Ending::Broke(BrowserError),
        Ending::Abandoned,
        Ending::Failed(RunError::Store(StoreError)),
    ] {
        let scene = &missing.scene;
        let ran = Fakes::of(scene).into_ran(Some(ending), None, 0);
        let stopped = QaWorld {
            ran: Some(ran),
            ..QaWorld::default()
        };
        assert_eq!(
            stopped.reason(),
            None,
            "a run that did not finish has no reason"
        );
        assert_eq!(stopped.when_beat(), Beat::End, "it has no beat but the end");
    }
}

/// Checks the beats of a log: every kind that is one.
pub fn check_beats() {
    let kinds = [
        "asked",
        "recovered",
        "stage_advanced",
        "run_finished",
        "decided",
    ];
    let beats: Vec<Option<Beat>> = kinds.iter().map(|kind| beat_of(kind)).collect();
    let expected = [
        Some(Beat::Ask),
        Some(Beat::Core),
        Some(Beat::Advance),
        Some(Beat::End),
        None,
    ];
    assert_eq!(beats, expected, "the beats of the log");
}

/// Checks the world: the run, and what the accessors read from it.
pub async fn check() {
    check_beats();
    check_stopped().await;
    check_configured().await;
}
