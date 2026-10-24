//! Steps of this Rule of the feature, and the scenarios that have them.

#[path = "modes_through.rs"]
mod through;

use cucumber::{given, then, when};
use serde_json::{Value, json};

use crate::input::{others, page, reset, say, settle, step, step_calls, trial_ending};
use crate::page::PageSpec;
use crate::scene::{EMAIL_RULE, found};
use crate::world::InterviewWorld;

/// The scenarios of this Rule that have their steps.
pub const READY: &[&str] = &[
    "インタビューのモードでも、既存のガードレールで止まる操作は実行しない",
    "インタビューのモードでは、引き返しに要る確信度を書かなければ 0.3 を使い、既存のモードで止まる back も実行する",
    "インタビューのモードでも、判断の回数が上限に達したら試行を止める",
    "インタビューのモードでも、設定で指定した機密情報は判断モデルに送るテキストで伏せる",
    "インタビューのモードは課題を直さず、台帳と実行結果のほかにファイルを書き換えない",
];

/// The scenarios of this Rule left to another test: that the acceptance
/// scenarios of the exploring and the scripted modes pass is what
/// `--test acceptance` checks in CI, so this binary does not run it again.
pub const DELEGATED: &[&str] = &["探索モードと決め打ちシナリオの受け入れシナリオは、そのまま通る"];

/// The start page shows the button `name` and a link "ヘルプ".
fn shows_button(world: &mut InterviewWorld, name: &str) {
    let scene = &mut world.scene;
    scene.start = PageSpec::at("/")
        .with(PageSpec::button(name))
        .with(PageSpec::link("ヘルプ"));
}

#[given(expr = "判断モデルが禁止要素 {string} の click を確信度 {float} で返す")]
fn clicks_a_forbidden_element(world: &mut InterviewWorld, name: String, confidence: f64) {
    shows_button(world, &name);
    let scene = &mut world.scene;
    scene.forbidden.push(name);
    say(scene, [step("click", confidence, Some("目的を果たす"))]);
}

#[given(expr = "判断モデルが back を確信度 {float} で返す")]
fn goes_back(world: &mut InterviewWorld, confidence: f64) {
    let scene = &mut world.scene;
    scene.start = page("/");
    say(scene, [step("back", confidence, Some("目的を果たす"))]);
}

#[given(expr = "判断モデルが {string} の click を返し、実行の直前に {string} が消える")]
fn clicks_what_goes_away(world: &mut InterviewWorld, clicked: String, gone: String) {
    assert_eq!(
        clicked, gone,
        "the element that goes away is the one clicked"
    );
    shows_button(world, &clicked);
    let scene = &mut world.scene;
    // The page just before the click no longer shows the button.
    scene.after = vec![vec![PageSpec::at("/").with(PageSpec::link("ヘルプ"))]];
    say(scene, [step("click", 0.9, Some("目的を果たす"))]);
    drop((clicked, gone));
}

/// Why the log says the steps were rejected, in order.
fn rejections(world: &InterviewWorld) -> Vec<Value> {
    let lines = world.lines();
    let rejected = lines
        .iter()
        .filter(|line| line.get("kind") == Some(&json!("rejected")));
    rejected
        .filter_map(|line| line.get("why").cloned())
        .collect()
}

#[then(expr = "ブラウザで {string} を操作していない")]
async fn did_not_operate(world: &mut InterviewWorld, name: String) {
    world.run().await;
    let ran = world.ran();
    assert!(!ran.judge.asked.is_empty(), "the judge was asked");
    let targets = ran
        .touches
        .iter()
        .map(|(_, touch)| touch.target.as_ref().map(|target| target.name.clone()));
    let operated = targets.filter(|target| target.as_ref() == Some(&name));
    assert_eq!(operated.count(), 0, "the element was left alone");
    // The element is forbidden, or it was gone when the step was to run.
    let forbidden = world.scene.forbidden.contains(&name);
    let why = if forbidden { "forbidden" } else { "missing" };
    assert_eq!(rejections(world), [json!(why)], "why the step was stopped");
}

#[then(expr = "ブラウザで 「戻る」 を操作していない")]
async fn did_not_go_back(world: &mut InterviewWorld) {
    world.run().await;
    let ran = world.ran();
    assert!(!ran.judge.asked.is_empty(), "the judge was asked");
    let backs = ran.touches.iter().filter(|(_, touch)| touch.op == "back");
    assert_eq!(backs.count(), 0, "the browser did not go back");
    let why = [json!("low_confidence")];
    assert_eq!(rejections(world), why, "why the step was stopped");
}

#[given(
    expr = "既存のモードの引き返しに要る確信度は 0.7 で、インタビューのモードの値は書いていない"
)]
const fn back_confidence_left_out(world: &mut InterviewWorld) {
    // The configuration of the scenes writes 0.7 in `limits`.
    world.scene.back_confidence = None;
}

#[then(expr = "ブラウザで 「戻る」 を操作し、止めていない")]
async fn went_back(world: &mut InterviewWorld) {
    world.run().await;
    let ran = world.ran();
    let backs = ran.touches.iter().filter(|(_, touch)| touch.op == "back");
    assert_eq!(backs.count(), 1, "the browser went back");
    assert_eq!(
        rejections(world),
        Vec::<Value>::new(),
        "nothing was stopped"
    );
}

#[given(
    expr = "止めた引き返しも行き詰まりに数えるので、インタビューのモードの行き詰まりの閾値は {int} 回に設定されている"
)]
const fn stuck_above_the_calls(world: &mut InterviewWorld, threshold: u32) {
    world.scene.interview_stuck = Some(threshold);
}

#[when(expr = "1回の試行で {int} 回判断する")]
async fn calls_the_judge(world: &mut InterviewWorld, calls: usize) {
    let scene = &mut world.scene;
    scene.start = page("/");
    // A back that the judge is not sure of stops the step: it uses one call
    // and nothing else. Each stop is a step without progress, so the
    // scenario sets the stuck threshold above the calls.
    let unsure = (0..calls).map(|_| step("back", 0.2, Some("目的を果たす")));
    reset(scene);
    say(scene, unsure);
    settle(scene);
    world.run().await;
}

#[then(expr = "その試行は判断予算切れとして終わっている")]
async fn trial_ran_out_of_budget(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(trial_ending(world, 1), json!("budget_spent"), "the ending");
    let calls = usize::try_from(world.scene.max_decisions).unwrap_or(0);
    let asked = calls + others(&world.scene);
    assert_eq!(
        step_calls(world),
        asked,
        "the calls of the trial, then of the others"
    );
}

#[then(expr = "判断モデルに渡したアクセシビリティ情報のテキストに {string} は無い")]
async fn text_has_no_address(world: &mut InterviewWorld, address: String) {
    world.run().await;
    let asked = &world.ran().judge.asked;
    let first = found(asked.first(), "the judge was asked");
    let shown = format!("1: link \"[[{EMAIL_RULE}]]\"");
    assert_eq!(first.screen, shown, "the address is hidden in the screen");
    assert!(!first.screen.contains(&address), "the address is not sent");
}
