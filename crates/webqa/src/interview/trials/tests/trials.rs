//! Tests of the trials the loop runs, and of the records it writes.

use super::*;

#[tokio::test]
async fn runs_configured_trials_into_one_ledger() {
    let mut world = World::new(3);
    let ending = world.run().await;
    let expected = Ledger {
        head: head(3),
        entries: Vec::new(),
        unchecked_purposes: Vec::new(),
    };
    let wrote = Ending::Wrote {
        stopped: Stopped::Completed,
        confirmed: false,
    };
    assert_eq!(
        (ending, world.ledger(), world.launch.launched),
        (wrote, Some(expected), 3),
        "three trials, one ledger"
    );
}

#[tokio::test]
async fn each_trial_opens_fresh_browser() {
    let mut world = World::new(2);
    world.run().await;
    let kinds = world.kinds();
    let kinds: Vec<&str> = kinds.iter().map(String::as_str).collect();
    let trial = ["trial_started", "asked", "trial_finished"];
    let expected = [&["run_started"][..], &trial, &trial, &["run_finished"]].concat();
    assert_eq!(
        (world.launch.launched, world.launch.browsers.len(), kinds),
        (2, 0, expected),
        "a browser each, and the lines of each trial between the first and last"
    );
}

#[tokio::test]
async fn one_report_for_the_whole_run() {
    let mut world = World::new(2);
    world.run().await;
    let report = world.store.report.unwrap();
    let calls = report["calls"].as_array().map(Vec::len);
    assert_eq!(
        (report["reason"].as_str(), calls),
        (Some("reported_done"), Some(2)),
        "the calls of both trials in one report"
    );
}

#[tokio::test]
async fn abort_mid_trial_skips_rest() {
    let mut world = World::new(3);
    let mut interrupt = Fires {
        calls: 0,
        fire_on: vec![2],
    };
    let ending = world.run_with(&mut interrupt).await;
    let interrupted = Ending::Wrote {
        stopped: Stopped::Interrupted,
        confirmed: false,
    };
    let ledger = world.ledger().map(|ledger| ledger.head);
    let last = world.store.lines.iter().rev().nth(1).cloned();
    let cut = serde_json::json!({
        "kind": "trial_finished", "trial": 2, "ending": "interrupted",
        "seq": 5, "elapsed_ms": 0,
    });
    assert_eq!(
        (ending, ledger, world.launch.launched, last),
        (interrupted, Some(head(1)), 2, Some(cut)),
        "the cut trial is not finished, and no third trial starts"
    );
}
