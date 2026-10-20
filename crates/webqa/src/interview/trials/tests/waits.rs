//! Tests of the loop waiting for the window, and of what stops it.

use super::*;

#[tokio::test]
async fn deadline_while_waiting_times_out() {
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
        (timed_out, Some(head(1)), 1, minutes(30)),
        "woke at the deadline, not at the next window, and started no more"
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
