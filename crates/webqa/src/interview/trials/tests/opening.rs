//! Tests of opening the browser of a trial.

use super::*;
use crate::exit::interview_code;

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
async fn browser_that_cannot_open_writes_records() {
    let mut world = World::new(1);
    world.launch.browsers.clear();
    let ending = world.run().await;
    let ledger = world.ledger().map(|ledger| ledger.head);
    let reason = world.store.report.as_ref().map(|r| r["reason"].clone());
    let last = world.kinds().last().cloned();
    assert_eq!(
        (ending, ledger, reason, last),
        (
            Ending::Broke(BrowserError),
            Some(head(0)),
            Some(serde_json::json!("error")),
            Some("run_finished".to_owned())
        ),
        "the records of no trial, ending in an error"
    );
}

#[tokio::test]
async fn browser_failure_in_second_trial_keeps_first() {
    let mut world = World::new(2);
    world.judge.script = script(&["done", "click", "click"]);
    // The second browser shows the first page and the looks before and
    // after one click, then fails.
    let second = world.launch.browsers.get_mut(1).unwrap();
    second.pages.truncate(3);
    let ending = world.run().await;
    let code = interview_code(&ending);
    let ledger = world.ledger().map(|ledger| ledger.head);
    let report = world.store.report.clone().unwrap();
    let ops: Vec<_> = report["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["op"].clone())
        .collect();
    assert_eq!(
        (ending, code, ledger, report["reason"].clone(), ops),
        (
            Ending::Broke(BrowserError),
            3,
            Some(head(1)),
            serde_json::json!("error"),
            vec![serde_json::json!("click")]
        ),
        "the ledger of the first trial, and the click of the second in the report"
    );
}
