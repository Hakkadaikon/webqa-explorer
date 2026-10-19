//! Steps of the Rule on recording the issues found.

use cucumber::{given, then, when};
use serde_json::{Value, json};
use webqa::store::screenshots;
use webqa_core::issue::IssueKind;

use crate::judge::Reply;
use crate::page::{ElementSpec, PageSpec};
use crate::world::QaWorld;

#[path = "issues_report.rs"]
mod report;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "問題を見つけたら、種類とステップ番号を含む名前でスクリーンショットを残す",
    "エラー表示の確信度が閾値ちょうどなら問題として記録する",
    "エラー表示の確信度が閾値に届かなければ問題として記録しない",
    "読み上げ名が画像ファイル名や仮の値の要素はアクセシビリティの問題として記録する",
    "ブラウザのコンソールにエラーが出たら問題として記録する",
    "意味の判断が要る検出は、判断モデルだけの判定なら参考情報として記録する",
    "実行結果は再実行できる形で出力する",
];

/// A button named `name`.
fn button(name: impl Into<String>) -> ElementSpec {
    PageSpec::button(&name.into())
}

/// The `issue` lines of the event log of the kind `kind`.
fn found_of(world: &QaWorld, kind: &str) -> Vec<Value> {
    let lines = world.lines();
    let issues = lines.into_iter().filter(|line| line["kind"] == "issue");
    issues.filter(|line| line["issue"] == kind).collect()
}

/// The page shown after the click on "保存", whose console says `error`.
fn click_save(world: &mut QaWorld) {
    world.scene.show_button("保存");
    world
        .scene
        .replies
        .push_back(Reply::op("click", 0.9).on("保存"));
}

/// The start page shows the buttons 1 to `before`; each click on one of them
/// shows the page with one more result, so that no step is stuck.
fn progress(world: &mut QaWorld, before: usize) {
    let names: Vec<String> = (1..=before).map(|n| format!("操作{n}")).collect();
    for name in &names {
        world.scene.show_button(name);
    }
    let clicks = names.iter().map(|name| Reply::op("click", 0.9).on(name));
    world.scene.replies.extend(clicks);
    results_shown(world, before);
}

/// Each click shows the start page with the result `n` added.
fn results_shown(world: &mut QaWorld, before: usize) {
    let results = (1..=before).map(|n| PageSpec::button(&format!("結果{n}")));
    let start = world.scene.start.clone();
    let pages = results.map(|result| start.clone().with(result));
    world.scene.after.extend(pages);
}

#[given(expr = "{int} ステップ目で判断モデルがエラー表示ありを確信度 {float} で返す")]
fn error_at(world: &mut QaWorld, nth: usize, p: f64) {
    progress(world, nth - 1);
    world
        .scene
        .replies
        .push_back(Reply::op("wait", 0.9).error(p));
}

#[given(expr = "エラー表示の判定閾値は {float} に設定されている")]
const fn threshold(world: &mut QaWorld, p: f64) {
    world.scene.error_confidence = p;
}

#[when(expr = "{int} ステップ目を実行する")]
async fn run_step(world: &mut QaWorld, nth: usize) {
    assert_eq!(
        world.scene.replies.len(),
        nth,
        "the step asked is the last queued"
    );
    world.run().await;
}

#[then(expr = "エラー表示の問題として記録されている")]
async fn error_recorded(world: &mut QaWorld) {
    world.run().await;
    assert!(
        !found_of(world, "error_display").is_empty(),
        "an error display issue"
    );
}

#[then(expr = "種類とステップ番号 {int} を含む名前のスクリーンショットが保存されている")]
async fn shot_saved(world: &mut QaWorld, nth: u64) {
    world.run().await;
    let name = screenshots::name(IssueKind::ErrorDisplay, nth);
    let names: Vec<&String> = world
        .ran()
        .store
        .shots
        .iter()
        .map(|(name, _)| name)
        .collect();
    assert!(
        names.contains(&&name),
        "the shot {name} was saved: {names:?}"
    );
}

#[given(expr = "判断モデルがエラー表示ありを確信度 {float} で返す")]
fn error_shown(world: &mut QaWorld, p: f64) {
    world
        .scene
        .replies
        .push_back(Reply::op("wait", 0.9).error(p));
}

#[then(expr = "エラー表示の問題は記録されていない")]
async fn error_not_recorded(world: &mut QaWorld) {
    world.run().await;
    assert_eq!(
        found_of(world, "error_display"),
        Vec::<Value>::new(),
        "no issue"
    );
}

#[given(expr = "対象ページに読み上げ名が {string} のボタンがある")]
fn nameless_button(world: &mut QaWorld, name: String) {
    world.scene.start = world.scene.start.clone().with(button(name));
}

#[then(expr = "アクセシビリティの問題として記録されている")]
async fn accessibility_recorded(world: &mut QaWorld) {
    world.run().await;
    assert!(
        !found_of(world, "accessible_name").is_empty(),
        "an accessibility issue"
    );
}

#[then(expr = "アクセシビリティの問題として記録されていない")]
async fn accessibility_not_recorded(world: &mut QaWorld) {
    world.run().await;
    assert_eq!(
        found_of(world, "accessible_name"),
        Vec::<Value>::new(),
        "no issue"
    );
}

#[given(expr = "操作のあとにブラウザのコンソールへエラーログが出力される")]
fn console_error(world: &mut QaWorld) {
    click_save(world);
    let mut after = world.scene.start.clone();
    after.console.push("TypeError: x is undefined".to_owned());
    world.scene.after.push_back(after);
}

#[then(expr = "コンソールエラーの問題として記録されている")]
async fn console_recorded(world: &mut QaWorld) {
    world.run().await;
    assert!(
        !found_of(world, "console_error").is_empty(),
        "a console error issue"
    );
}

#[given(expr = "判断モデルが「わかりにくい表示」があると返す")]
fn confusing(world: &mut QaWorld) {
    world.scene.probes = true;
    world
        .scene
        .replies
        .push_back(Reply::op("wait", 0.9).confusing(0.9));
}

#[then(expr = "その検出は参考情報として記録され、確定した不具合とは別に分類されている")]
async fn reference_recorded(world: &mut QaWorld) {
    world.run().await;
    let found = found_of(world, "semantic");
    let described: Vec<(&Value, &Value)> =
        found.iter().map(|l| (&l["class"], &l["label"])).collect();
    assert_eq!(
        json!(described),
        json!([["reference", "わかりにくい表示"]]),
        "a reference finding"
    );
    let issues = world
        .lines()
        .into_iter()
        .filter(|line| line["kind"] == "issue");
    let confirmed = issues.filter(|line| line["class"] == "confirmed");
    assert_eq!(confirmed.count(), 0, "nothing confirmed");
}
