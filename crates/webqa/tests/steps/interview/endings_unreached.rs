//! The trials that do not reach the purpose, and those that do not count as
//! such.

use cucumber::{given, then};
use webqa_core::interview::issue::Kind;

use super::{goal_at_start, says};
use crate::questions::replies::{ledger_of, step};
use crate::world::InterviewWorld;

#[given(expr = "試行が 行き詰まり として終わる")]
fn ends_stuck(world: &mut InterviewWorld) {
    says(world, step("stuck", None, 0.0));
}

#[given(expr = "試行が 判断予算切れ として終わる")]
fn ends_over_budget(world: &mut InterviewWorld) {
    world.scene.max_decisions = 1;
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

/// The numbers of finished trials and of those that did not reach the
/// purpose, of each issue of the ledger about a purpose not reached.
fn unreached(world: &InterviewWorld) -> Vec<(usize, usize)> {
    let ledger = ledger_of(world);
    let issues = ledger.entries.iter().map(|entry| &entry.issue);
    let not_reached = issues.filter(|issue| issue.kind() == Kind::Unreached);
    not_reached
        .map(|issue| (issue.count.occurred, issue.count.finished))
        .collect()
}

#[then(expr = "その試行は目的に届かない試行に数えられている")]
async fn counted_unreached(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(
        unreached(world),
        [(3, 3)],
        "all three trials did not reach it"
    );
}

#[then(expr = "その試行は目的に届かない試行に数えられていない")]
async fn not_counted_unreached(world: &mut InterviewWorld) {
    world.run().await;
    let ledger = ledger_of(world);
    assert_eq!(ledger.head.finished, 3, "the trials finished");
    assert_eq!(unreached(world), [], "none is counted as not reaching it");
}
