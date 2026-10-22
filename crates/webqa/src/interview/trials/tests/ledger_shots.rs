//! Tests of the step the ledger names for an issue.

use std::slice;

use webqa_core::interview::answer::Reply;
use webqa_core::interview::clue::Thresholds;
use webqa_core::interview::issue::Shot;
use webqa_core::interview::outcome::{self, Outcome};
use webqa_core::judge::{Usage, Verdict};
use webqa_core::machine::ExitReason;

use super::super::record::{Trial, steps};
use super::super::write::{Closing, records};
use super::shots::{photographed, written};
use super::*;
use crate::run::fakes::PNG;
use crate::run::{Kept, Taken};
use crate::store::screenshots::issue_name;

/// A step of the log's number `step` where the judge hesitated.
fn hesitant(step: u64) -> Taken {
    let verdict = Verdict {
        op: Op::Click,
        confidence: Some(0.9),
        targets: Vec::new(),
        error_shown: None,
        goal_reached: None,
        usage: Usage::default(),
    };
    let reply = Reply {
        verdict,
        hesitation: 0.9,
        expectation: String::new(),
        hard_to_see: None,
        purpose_guess: String::new(),
    };
    Taken {
        reply: Some(reply),
        ..photographed(step, &PNG)
    }
}

/// A trial whose second step is the log's step 3, where the judge hesitated.
fn trial() -> Trial {
    let thresholds = Thresholds {
        hesitation: 0.5,
        confidence: 0.5,
    };
    Trial {
        outcome: Outcome::Finished(outcome::Ending::Goal),
        reason: ExitReason::GoalAchieved,
        steps: steps(vec![photographed(1, &PNG), hesitant(3)], &[], thresholds),
    }
}

/// Writes the records of `trial` into the world's store.
async fn close(world: &mut World, trial: &Trial) -> bool {
    let mut kept = Kept::default();
    let closing = Closing {
        judge: &mut world.judge,
        clock: &mut world.clock,
        store: &mut world.store,
        redactor: &world.redactor,
        kept: &mut kept,
        trials: slice::from_ref(trial),
        plan: &world.plan,
        reset_failed: false,
    };
    records(closing, Stopped::Completed).await.is_ok()
}

#[tokio::test]
async fn the_ledger_names_the_log_step_of_the_issue() {
    let mut world = World::new(1);
    assert!(close(&mut world, &trial()).await, "the records are written");
    let issue = world.ledger().unwrap().entries[0].issue.clone();
    let shot = Shot { trial: 1, step: 3 };
    let name = issue_name(issue.number, shot);
    assert_eq!(
        (
            issue.shot,
            name.ends_with("_trial1_step3.png"),
            written(&world)
        ),
        (Some(shot), true, vec![(name, PNG.to_vec())]),
        "the log's step, not the position"
    );
}
