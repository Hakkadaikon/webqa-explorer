//! The Given steps of the inputs that the configuration rejects, and the
//! Then step that nothing was opened.

use cucumber::{given, then};
use serde_json::json;

use super::reject::persona_doc;
use crate::world::InterviewWorld;

/// The persona document without the item `key`.
fn without(world: &mut InterviewWorld, key: &str) {
    let doc = persona_doc(&world.scene.persona, Some(key));
    world.scene.persona_text = Some(doc);
}

#[given(expr = "ペルソナの文書に 名前 が書かれていない")]
fn no_name(world: &mut InterviewWorld) {
    without(world, "name");
}

#[given(expr = "ペルソナの文書に どんな人か が書かれていない")]
fn no_profile(world: &mut InterviewWorld) {
    without(world, "profile");
}

#[given(expr = "ペルソナの文書に ITの慣れ が書かれていない")]
fn no_literacy(world: &mut InterviewWorld) {
    without(world, "it_literacy");
}

#[given(expr = "ペルソナの文書に このサービスで達成したいこと が書かれていない")]
fn no_wants(world: &mut InterviewWorld) {
    without(world, "wants");
}

#[given(expr = "ペルソナの文書に 知らないこと、しないこと が書かれていない")]
fn no_avoids(world: &mut InterviewWorld) {
    without(world, "avoids");
}

#[given(expr = "ペルソナの文書の書式が崩れていて、項目を読み取れない")]
fn broken_persona(world: &mut InterviewWorld) {
    let name = &world.scene.persona.name;
    world.scene.persona_text = Some(format!("name: [{name}\n"));
}

#[then(expr = "判断モデルを呼び出しておらず、対象ページも開いていない")]
async fn nothing_was_opened(world: &mut InterviewWorld) {
    world.run().await;
    let ran = world.ran();
    assert!(ran.judge.asked.is_empty(), "the judge was not asked");
    assert!(ran.bridge.is_empty(), "the bridge was not asked");
    assert_eq!(ran.launched, 0, "no browser was opened");
}

#[given(expr = "画面ごとの目的の URL の型が {string} である")]
fn pattern_is(world: &mut InterviewWorld, pattern: String) {
    let screens = json!({ "screens": [{ "url": pattern, "purpose": "商品をカートに入れる" }] });
    world.scene.purpose_text = Some(screens.to_string());
    drop(pattern);
}

#[given(expr = "設定の 試行の回数 が {int} である")]
const fn trials_are(world: &mut InterviewWorld, trials: u32) {
    world.scene.trials = trials;
}

#[given(expr = "設定の 迷いの度合いの閾値 が {float} である")]
const fn hesitation_is(world: &mut InterviewWorld, threshold: f64) {
    world.scene.hesitation = threshold;
}

#[given(expr = "設定の 迷いとみなす確信度の閾値 が {float} である")]
const fn confidence_is(world: &mut InterviewWorld, threshold: f64) {
    world.scene.confidence = threshold;
}

#[given(expr = "設定の インタビューのモードの行き詰まりの閾値 が {int} である")]
const fn interview_stuck_is(world: &mut InterviewWorld, threshold: u32) {
    world.scene.interview_stuck = Some(threshold);
}

#[given(expr = "設定の 自然さの問いの回数の上限 が {int} である")]
const fn natural_questions_are(world: &mut InterviewWorld, cap: u32) {
    world.scene.max_natural = cap;
}

#[given(expr = "設定の インタビューのモードの1時間窓あたりのアクセスの上限 が {int} である")]
const fn interview_requests_are(world: &mut InterviewWorld, cap: u32) {
    world.scene.interview_requests = Some(cap);
}
