//! Tests of the loop waiting for the window, and of what stops it.

use webqa_core::ports::StoreError;

use super::*;

#[tokio::test]
async fn a_window_opening_after_the_deadline_times_out_at_once() {
    let mut world = World::tight(2);
    world.plan.max_duration = Some(minutes(30));
    let ending = world.run().await;
    let timed_out = Ending::Wrote {
        stopped: Stopped::TimeUp,
        confirmed: false,
    };
    let ledger = world.ledger().map(|ledger| ledger.head);
    assert_eq!(
        (ending, ledger, world.launch.launched, world.clock.now),
        (timed_out, Some(head(1)), 1, Duration::ZERO),
        "the next window opens after the deadline: no wait, no more trials"
    );
}

#[tokio::test]
async fn abort_while_waiting_writes_ledger() {
    let mut world = World::tight(2);
    let mut clock = Stalled {
        now: Duration::ZERO,
        sleeps: Vec::new(),
    };
    // The first trial waits on the interruption to open the browser and
    // for its step: the wait for the window is the third.
    let mut interrupt = Fires {
        calls: 0,
        fire_on: vec![3],
    };
    let ending = watched(interview(
        &mut world.launch,
        &mut world.judge,
        &mut clock,
        &mut world.store,
        &world.redactor,
        &mut interrupt,
        &world.plan,
    ))
    .await;
    let interrupted = Ending::Wrote {
        stopped: Stopped::Interrupted,
        confirmed: false,
    };
    let ledger = world.ledger().map(|ledger| ledger.head);
    assert_eq!(
        (ending, ledger, world.launch.launched, clock.sleeps),
        (interrupted, Some(head(1)), 1, vec![minutes(60)]),
        "the wait for the next window ended with the interruption"
    );
}

#[tokio::test]
async fn the_wait_between_trials_is_logged() {
    let mut world = World::tight(2);
    world.run().await;
    let kinds = world.picked(&["trial_finished", "waited", "trial_started"]);
    let order = [
        "trial_started",
        "trial_finished",
        "waited",
        "trial_started",
        "trial_finished",
    ];
    assert_eq!(kinds, order, "the wait between the two trials");
    let waited = world
        .store
        .lines
        .iter()
        .find(|line| line["kind"] == "waited");
    let times = waited.map(|line| (line["elapsed_ms"].as_u64(), line["until_ms"].as_u64()));
    let want = (Some(0), Some(3_600_000));
    assert_eq!(
        times,
        Some(want),
        "written before sleeping to the next window"
    );
}

#[tokio::test]
async fn a_wait_that_cannot_be_logged_fails_the_run() {
    let mut logged = World::tight(2);
    logged.run().await;
    let kinds = logged.kinds();
    let before = kinds.iter().position(|kind| kind == "waited");
    let mut world = World::tight(2);
    world.store.limit = before;
    let ending = world.run().await;
    // The store keeps a line even when its write fails, and an earlier
    // failed write would stop the run before the waited line.
    let failed = Ending::Failed(RunError::Store(StoreError));
    let waited = world.kinds().contains(&"waited".to_owned());
    let state = (ending, waited, world.launch.launched, world.clock.now);
    let want = (failed, true, 1, Duration::ZERO);
    assert_eq!(
        state, want,
        "the waited line failed: no wait, no second trial"
    );
}
