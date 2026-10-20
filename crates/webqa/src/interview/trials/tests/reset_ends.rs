//! Tests of how a failing reset command ends the run, and of what it gets.

use super::resets::Watching;
use super::*;
use crate::interview::reset::ResetError;

#[tokio::test]
async fn failed_reset_before_any_trial_is_a_run_that_got_nowhere() {
    let allow = [var("RESET_ALLOW", "0")];
    let mut world = World::resetting(2, "failed_reset_first", "counts.sh", &allow);
    let (ending, _) = Watching::run(&mut world).await;
    let report = world.store.report.unwrap();
    assert_eq!(
        (
            ending,
            report["reason"].clone(),
            report["stuck"].clone(),
            world.launch.launched
        ),
        (
            Ending::ResetFailed(ResetError::Exit(1)),
            serde_json::json!("stuck"),
            serde_json::json!("no_progress"),
            0
        ),
        "the report does not say the user stopped it"
    );
}

#[tokio::test]
async fn reset_failure_writes_ledger_then_exits_three() {
    let allow = [var("RESET_ALLOW", "1")];
    let mut world = World::resetting(3, "reset_failure_writes_ledger", "counts.sh", &allow);
    let (ending, _) = Watching::run(&mut world).await;
    let ledger = world.ledger().map(|ledger| ledger.head);
    let report = world.store.report.unwrap();
    assert_eq!(
        (
            ending,
            ledger,
            world.launch.launched,
            report["reason"].clone()
        ),
        (
            Ending::ResetFailed(ResetError::Exit(1)),
            Some(head(1)),
            1,
            serde_json::json!("reported_done")
        ),
        "the ledger of the first trial, and the reason of that trial"
    );
}

#[tokio::test]
async fn reset_child_env_withholds_api_key() {
    let key = "value-of-the-judge-key";
    let env = [
        var("COPY_OF_KEY", format!("Bearer {key}")),
        var("KEEP", "kept"),
    ];
    let mut world = World::resetting(1, "reset_child_env", "record.sh", &env);
    world.plan.withheld = vec![key.to_owned()];
    Watching::run(&mut world).await;
    let seen = fs::read_to_string(world.out().with_extension("env")).unwrap();
    assert_eq!(
        (
            seen.contains(key),
            seen.contains("COPY_OF_KEY"),
            seen.lines().any(|line| line == "KEEP=kept")
        ),
        (false, false, true),
        "the variable with the key is left out and the others are passed"
    );
}
