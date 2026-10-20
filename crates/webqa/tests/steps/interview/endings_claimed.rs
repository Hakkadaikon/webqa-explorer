//! The trial the judge ends: done, or stuck.

use cucumber::{then, when};
use webqa_core::interview::issue::{Detail, Halt};
use webqa_core::interview::ledger::Goal;

use super::{endings, says};
use crate::questions::replies::{ledger_of, step};
use crate::world::InterviewWorld;

#[when(expr = "判断モデルが done を返す")]
async fn judge_done(world: &mut InterviewWorld) {
    says(world, step("done", None, 0.0));
    world.run().await;
}

#[when(expr = "判断モデルが stuck を返す")]
async fn judge_stuck(world: &mut InterviewWorld) {
    says(world, step("stuck", None, 0.0));
    world.run().await;
}

#[then(expr = "その試行は未検証の終了として終わっている")]
async fn trial_unverified(world: &mut InterviewWorld) {
    world.run().await;
    let trial = world.scene.trial.saturating_sub(1);
    assert_eq!(
        endings(world).get(trial),
        Some(&"done".to_owned()),
        "the ending"
    );
}

#[then(expr = "台帳に、推測した目的に届いたという申告として記録されている")]
async fn claim_recorded(world: &mut InterviewWorld) {
    world.run().await;
    let ledger = ledger_of(world);
    let guessed = Goal::Inferred(vec!["買い物をする".to_owned()]);
    assert_eq!(ledger.head.goal, guessed, "the purpose the persona claimed");
    assert_eq!(ledger.head.finished, 3, "the trials that finished");
}

#[then(expr = "その試行はゴール達成に数えられていない")]
async fn trial_not_goal(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(endings(world), ["done"; 3], "no trial reached the goal");
}

#[then(expr = "その試行は行き詰まりとして終わっている")]
async fn trial_stuck(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(endings(world), ["gave_up"; 3], "the persona gave up");
}

#[then(expr = "台帳に、ペルソナが諦めたことが記録されている")]
async fn gave_up_recorded(world: &mut InterviewWorld) {
    world.run().await;
    let ledger = ledger_of(world);
    let halts: Vec<&Detail> = ledger.entries.iter().map(|e| &e.issue.detail).collect();
    let gave_up = Detail::Unreached {
        halt: Halt::GaveUp,
        expectation: Some("進む".to_owned()),
    };
    assert_eq!(halts, [&gave_up], "the issue of the persona who gave up");
}
