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

/// The judge clicks without hesitating, then says it is done, hesitating.
fn click_then_hesitant_done() -> Scripted {
    let ops = [("click", 0.0), ("done", 0.9)];
    ops.iter()
        .fold(Scripted::default(), |script, &(op, hesitation)| {
            script
                .then(QuestionId::operation(), [(op, 0.9)])
                .then(QuestionId::target(), [("1", 0.8)])
                .then(QuestionId::error_shown(), [("true", 0.0)])
                .then(QuestionId::goal_reached(), [("true", 0.0)])
                .then(QuestionId::new(HESITATION), [("true", hesitation)])
                .then_text(QuestionId::new(EXPECTATION), "保存される")
                .then_text(QuestionId::new(PURPOSE_GUESS), GUESS)
        })
}

/// The steps of the log's lines of `kind`, each once.
fn logged_steps(world: &World, kind: &str) -> Vec<u64> {
    let lines = world.store.lines.iter();
    let lines = lines.filter(|line| line["kind"] == kind);
    let mut steps: Vec<u64> = lines.filter_map(|line| line["step"].as_u64()).collect();
    steps.dedup();
    steps
}

#[tokio::test]
async fn a_recovery_keeps_the_ledger_step_on_the_page_the_log_asked_about() {
    let mut world = World::new(1);
    // The page never changes, so the core reloads it after the click.
    world.plan.rules.freeze_threshold = nz(1);
    world.judge.script = click_then_hesitant_done();
    world.run().await;
    let recovered = logged_steps(&world, "recovered").len();
    assert_eq!(recovered, 1, "the core recovered at one step");
    let steps = logged_steps(&world, "asked");
    assert_eq!(steps.len(), 2, "the judge was asked at two steps");
    // The hesitation is at the second of them.
    let logged = steps[1];
    // Asked at 1, recovered at 2: the second question is the log's step 3,
    // not the 2nd of the questions.
    assert_eq!(logged, 3, "the recovery moved the log's step");
    let issue = world.ledger().unwrap().entries[0].issue.clone();
    let shot = issue.shot.unwrap();
    let name = issue_name(issue.number, shot);
    let suffix = format!("_trial1_step{logged}.png");
    assert_eq!(
        (shot.step, name.ends_with(&suffix), written(&world)),
        (logged, true, vec![(name, PNG.to_vec())]),
        "the ledger and its screenshot name the log's step of the hesitant page"
    );
}
