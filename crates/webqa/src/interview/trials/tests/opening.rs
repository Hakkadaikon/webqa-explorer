//! Tests of opening the browser of a trial.

use super::*;
use crate::run::RunError;

#[tokio::test]
async fn deadline_during_browser_start_cuts_without_waiting() {
    let mut world = World::new(2);
    world.launch.hangs = true;
    world.plan.max_duration = Some(minutes(30));
    let ending = world.run().await;
    let timed_out = Ending::Wrote {
        stopped: Stopped::TimeUp,
        confirmed: false,
    };
    let started = world.kinds().contains(&"trial_started".to_owned());
    assert_eq!(
        (ending, world.launch.launched, started, world.clock.now),
        (timed_out, 1, false, minutes(30)),
        "the launch that never ends does not hold the run past the deadline"
    );
}

#[tokio::test]
async fn interrupt_during_browser_start_cuts() {
    let mut world = World::new(2);
    world.launch.hangs = true;
    let mut interrupt = Fires {
        calls: 0,
        fire_on: vec![1],
    };
    let ending = world.run_with(&mut interrupt).await;
    let interrupted = Ending::Wrote {
        stopped: Stopped::Interrupted,
        confirmed: false,
    };
    assert_eq!(
        (ending, world.launch.launched),
        (interrupted, 1),
        "the interruption ends the launch that never ends"
    );
}

#[tokio::test]
async fn browser_that_cannot_open_fails_the_run() {
    let mut world = World::new(1);
    world.launch.browsers.clear();
    let ending = world.run().await;
    assert_eq!(
        (ending, world.store.ledger),
        (Ending::Failed(RunError::Browser(BrowserError)), None),
        "no ledger for a run that failed"
    );
}
