//! Answers to the added questions that cannot be used: the step is an
//! anomaly, and nothing is done in the browser.

use cucumber::{given, then};
use serde_json::Value;
use webqa_core::interview::ask::{EXPECTATION, HARD_TO_SEE, HESITATION, PURPOSE_GUESS};
use webqa_core::judge::Answers;

use super::replies::{SAVE, button, each_trial, step, without, written};
use crate::world::InterviewWorld;

/// A page with a button, and `answers` as the step of every trial.
fn answering(world: &mut InterviewWorld, answers: Answers) {
    button(&mut world.scene, SAVE);
    each_trial(&mut world.scene, &[answers]);
}

/// The answers of a step that clicks, without the answer to `id`.
fn lacking(id: &str) -> Answers {
    without(step("click", Some("1"), 0.0), id)
}

#[given(expr = "判断モデルの応答に 迷った度合い が無い")]
fn no_hesitation(world: &mut InterviewWorld) {
    answering(world, lacking(HESITATION));
}

#[given(expr = "判断モデルの応答に 期待の文 が無い")]
fn no_expectation(world: &mut InterviewWorld) {
    answering(world, lacking(EXPECTATION));
}

#[given(expr = "判断モデルの応答に 見えにくい要素の答え が無い")]
fn no_hard_to_see(world: &mut InterviewWorld) {
    world.scene.images = true;
    answering(world, lacking(HARD_TO_SEE));
}

#[given(expr = "判断モデルの応答に 画面の目的の推測 が無い")]
fn no_purpose_guess(world: &mut InterviewWorld) {
    answering(world, lacking(PURPOSE_GUESS));
}

/// The answers of a step that clicks, with the hesitation `p`.
fn hesitating(p: f64) -> Answers {
    step("click", Some("1"), p)
}

#[given(expr = "判断モデルの応答の 迷った度合い が 1.2 である")]
fn hesitation_over(world: &mut InterviewWorld) {
    answering(world, hesitating(1.2));
}

#[given(expr = "判断モデルの応答の 迷った度合い が -0.1 である")]
fn hesitation_under(world: &mut InterviewWorld) {
    answering(world, hesitating(-0.1));
}

#[given(expr = "判断モデルの応答の 見えにくい要素の番号 が 送った一覧に無い番号 99 である")]
fn hard_to_see_unlisted(world: &mut InterviewWorld) {
    world.scene.images = true;
    let answers = written(step("click", Some("1"), 0.0), HARD_TO_SEE, "99: 目立たない");
    answering(world, answers);
}

/// The calls of the first step of the first trial that the run counted.
pub(super) fn first_step_calls(world: &InterviewWorld) -> usize {
    let asked = Value::from("asked");
    let started = Value::from("trial_started");
    let lines = world.lines();
    let first = lines
        .iter()
        .skip_while(|line| line.get("kind") != Some(&started))
        .skip(1)
        .take_while(|line| line.get("kind") != Some(&started));
    let counted = |line: &&Value| {
        line.get("kind") == Some(&asked) && line.get("step") == Some(&Value::from(1))
    };
    first.filter(counted).count()
}

#[then(expr = "判断モデルの呼び出し回数は {int} 増えている")]
async fn calls_grew(world: &mut InterviewWorld, calls: usize) {
    world.run().await;
    assert_eq!(first_step_calls(world), calls, "the calls the step counted");
}
