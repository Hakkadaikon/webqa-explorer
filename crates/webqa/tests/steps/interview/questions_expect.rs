//! The second call of a step, which asks what the persona expects of the
//! operation the core carries out.

use cucumber::{given, then};
use webqa_core::interview::ask::EXPECTATION;

use super::asked::sent;
use super::replies::{
    CALM, SAVE, button, each_trial, first_call, natural_yes, prompt, reply, step,
};
use super::unusable::first_step_calls;
use crate::judge::{Asked, Kind};
use crate::scene::found;
use crate::world::InterviewWorld;

#[then(expr = "期待は、1回目の問い合わせでは問うていない")]
async fn first_call_has_no_expectation(world: &mut InterviewWorld) {
    world.run().await;
    let ids = ids_of(first_call(world));
    assert!(!ids.contains(&EXPECTATION.to_owned()), "{ids:?}");
}

/// The id of each question of `asked`, in order.
fn ids_of(asked: &Asked) -> Vec<String> {
    sent(asked).into_iter().map(|(id, _)| id).collect()
}

/// The second call of the judge of the scene.
fn second_call(world: &InterviewWorld) -> &Asked {
    found(world.ran().judge.asked.get(1), "the judge was asked twice")
}

#[then(expr = "判断モデルに、期待だけを2回目の問い合わせで問うている")]
async fn asks_expectation_alone(world: &mut InterviewWorld) {
    world.run().await;
    let second = sent(second_call(world));
    let only = vec![(EXPECTATION.to_owned(), Kind::Text)];
    assert_eq!(second, only, "the expectation alone, as a sentence");
    assert_eq!(first_step_calls(world), 2, "the step made two calls");
}

#[then(expr = "期待の問いの文に、操作 {string} と対象 {string} が入っている")]
async fn names_operation_and_target(world: &mut InterviewWorld, op: String, target: String) {
    world.run().await;
    let question = found(second_call(world).questions.first(), "a question");
    let prompt = prompt(question);
    let lines: Vec<&str> = prompt.lines().collect();
    let ask = format!("あなたは次に『{op}』をする。対象: {target}。");
    assert!(
        lines.iter().any(|line| line.starts_with(&ask)),
        "the sentence names both: {lines:?}"
    );
}

#[then(expr = "2回目の問い合わせには、1回目と同じ画面のテキストを渡している")]
async fn second_call_sees_the_same_screen(world: &mut InterviewWorld) {
    world.run().await;
    let (first, second) = (first_call(world), second_call(world));
    assert_eq!(second.screen, first.screen, "the same screen text");
}

#[given(expr = "判断モデルが操作として click を選ぶ")]
fn chooses_click(world: &mut InterviewWorld) {
    button(&mut world.scene, SAVE);
    each_trial(&mut world.scene, &[step("click", Some("1"), CALM)]);
}

#[given(expr = "判断モデルが操作として stuck を選ぶ")]
fn chooses_stuck(world: &mut InterviewWorld) {
    button(&mut world.scene, SAVE);
    each_trial(&mut world.scene, &[step("stuck", None, CALM)]);
    // The persona gave up: one issue, and the question about it.
    reply(&mut world.scene, natural_yes());
}

#[given(expr = "判断モデルが操作として done を選ぶ")]
fn chooses_done(world: &mut InterviewWorld) {
    button(&mut world.scene, SAVE);
    each_trial(&mut world.scene, &[step("done", None, CALM)]);
}

#[then(expr = "判断モデルへの問い合わせは {int} 回だけで、期待は問うていない")]
async fn asks_no_expectation(world: &mut InterviewWorld, calls: usize) {
    world.run().await;
    let step = world.ran().judge.asked.iter().take(first_step_calls(world));
    let expecting = step
        .clone()
        .filter(|call| ids_of(call).contains(&EXPECTATION.to_owned()));
    assert_eq!((step.count(), expecting.count()), (calls, 0), "asked once");
}
