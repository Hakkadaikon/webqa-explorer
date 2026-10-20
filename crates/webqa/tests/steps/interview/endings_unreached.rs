//! The trials that do not reach the purpose, and those that do not count as
//! such.

use cucumber::{given, then};

use super::{endings, goal_at_start, says, unreached_counts};
use crate::questions::replies::{SAVE, button, ledger_of, step};
use crate::world::InterviewWorld;

#[given(expr = "試行が 行き詰まり として終わる")]
fn ends_stuck(world: &mut InterviewWorld) {
    says(world, step("stuck", None, 0.0));
}

#[given(expr = "試行が 判断予算切れ として終わる")]
fn ends_over_budget(world: &mut InterviewWorld) {
    world.scene.max_decisions = 1;
    button(&mut world.scene, SAVE);
    says(world, step("click", Some("1"), 0.0));
}

#[given(expr = "試行が ゴール達成 として終わる")]
fn ends_goal(world: &mut InterviewWorld) {
    goal_at_start(&mut world.scene);
}

#[given(expr = r"試行が 推測した目的に届いたという申告\(未検証の終了\) として終わる")]
fn ends_claimed(world: &mut InterviewWorld) {
    says(world, step("done", None, 0.0));
}

#[then(expr = "その試行は目的に届かない試行に数えられている")]
async fn counted_unreached(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(endings(world), ["gave_up"; 3], "how the trials ended");
    assert_eq!(
        unreached_counts(world),
        [(3, 3)],
        "all three trials did not reach it"
    );
}

#[then(expr = "その試行は目的に届かない試行に数えられていない")]
async fn not_counted_unreached(world: &mut InterviewWorld) {
    world.run().await;
    let ledger = ledger_of(world);
    assert_eq!(ledger.head.finished, 3, "the trials finished");
    assert_eq!(
        unreached_counts(world),
        [],
        "none is counted as not reaching it"
    );
}

#[then(expr = "その試行は判断予算切れとして試行の数に数えられ、目的に届かない課題は出ていない")]
async fn budget_counted_not_unreached(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(endings(world), ["budget_spent"; 3], "how the trials ended");
    assert_eq!(ledger_of(world).head.finished, 3, "the trials are in N");
    assert_eq!(
        unreached_counts(world),
        [],
        "no issue of an unreached purpose"
    );
}
