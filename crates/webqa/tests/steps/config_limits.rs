//! Steps of the Rule of the configuration: the output, the errors and the
//! limits. The scenarios whose steps are all here are named in `READY`.

use std::iter::once;
use std::time::Duration;

use cucumber::{given, then, when};
use webqa_core::event::Mode;

use crate::judge::Reply;
use crate::scene::found;
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "実行結果の出力とログでも、指定した機密情報を伏せる",
    "設定ファイルに API キーが直接書かれていたら設定エラーにする",
    "参照先の環境変数に API キーが無ければ起動しない",
    "注入する認証状態の形式が不正なら設定エラーにする",
    "0 以下の閾値・上限や空の操作種別は設定エラーにする",
    "探索モードで使う操作種別に click も input も無ければ設定エラーにする",
    "判断・操作・画面変化待ちの時間と、判断モデルのコストを記録する",
];

/// What the YAML says for each item of the Examples: the line of the base,
/// and what replaces it, where `{}` is the value.
const ITEMS: [(&str, &str, &str); 7] = [
    (
        "固まり判定の閾値",
        "freeze_threshold: 3",
        "freeze_threshold: {}",
    ),
    (
        "行き詰まり判定の閾値",
        "stuck_threshold: 3",
        "stuck_threshold: {}",
    ),
    (
        "判断モデルの呼び出し上限",
        "max_decisions: 20",
        "max_decisions: {}",
    ),
    (
        "1時間窓あたりの操作回数の上限",
        "requests_per_hour: 10",
        "requests_per_hour: {}",
    ),
    (
        "ページの応答の待ち時間(秒)",
        "page_timeout_secs: 10",
        "page_timeout_secs: {}",
    ),
    (
        "判断モデルの応答の待ち時間(秒)",
        "timeout_secs: 30",
        "timeout_secs: {}",
    ),
    (
        "使う操作種別",
        "        url: /home\n",
        "        url: /home\nops: {}\n",
    ),
];

#[when(expr = "1ステップを実行し、実行結果を出力する")]
async fn step_and_report(world: &mut QaWorld) {
    world
        .scene
        .replies
        .push_back(Reply::op("click", 0.9).on_number(1));
    world.run().await;
}

#[then(expr = "実行結果の出力とログに {string} は含まれない")]
async fn outputs_hide(world: &mut QaWorld, value: String) {
    world.run().await;
    let report = found(world.report(), "the report was written").to_string();
    let lines = world.lines();
    let lines = lines.iter().map(ToString::to_string);
    let outputs: Vec<String> = once(report).chain(lines).collect();
    let leaks = outputs.iter().filter(|text| text.contains(&value));
    assert_eq!(leaks.count(), 0, "the report and the log hide it");
    drop(value);
}

#[then(expr = "イベントログに {string} は含まれない")]
async fn log_hides(world: &mut QaWorld, value: String) {
    world.run().await;
    let lines = world.lines();
    let leaks = lines
        .iter()
        .filter(|line| line.to_string().contains(&value));
    assert_eq!(leaks.count(), 0, "the log has no value of an input");
}

#[then(expr = "エラーの表示に API キーの値は含まれない")]
async fn error_hides_key(world: &mut QaWorld) {
    world.run().await;
    let sentence = world.ran().config_error.clone().unwrap_or_default();
    assert!(!sentence.contains("sk-live-123"), "the key is hidden");
}

#[then(expr = "起動は設定エラーで失敗し、判断モデルを呼び出していない")]
async fn fails_without_asking(world: &mut QaWorld) {
    world.run().await;
    let ran = world.ran();
    assert_ne!(ran.config_error, None, "the configuration was rejected");
    assert_eq!((ran.code, world.asked()), (2, 0), "code 2, no question");
}

#[given(expr = "設定のセッション情報が読み込めない形式である")]
fn broken_session(world: &mut QaWorld) {
    world.configure("storage_state: ./auth.json", "storage_state: ./broken.json");
}

#[given(expr = "設定の {string} が {string} である")]
fn item_is(world: &mut QaWorld, item: String, value: String) {
    let known = ITEMS.iter().find(|(name, ..)| *name == item);
    let (_, from, to) = found(known, "the item is one of the table");
    world.configure(from, &to.replace("{}", &value));
    drop((item, value));
}

#[then(expr = "同じ設定で決め打ちシナリオを起動しようとしても、設定エラーにならない")]
async fn scenario_accepts(world: &mut QaWorld) {
    world.ran = None;
    world.scene.mode = Mode::Scenario;
    world.run().await;
    assert_eq!(world.ran().config_error, None, "the scenario starts");
}

#[when(expr = "{int} ステップを実行して終了する")]
async fn steps_and_end(world: &mut QaWorld, steps: u32) {
    world.scene.think = Duration::from_millis(200);
    world.scene.limits.insert("max_decisions", steps);
    for name in ["保存", "ヘルプ", "設定"] {
        world.scene.show_button(name);
        world
            .scene
            .replies
            .push_back(Reply::op("click", 0.9).on(name));
    }
    world.run().await;
}

#[then(expr = "実行記録にステップごとの判断・操作・画面変化待ちの時間が含まれる")]
async fn timings(world: &mut QaWorld) {
    world.run().await;
    let report = found(world.report(), "the report was written");
    let rows = |name: &str| report[name].as_array().cloned().unwrap_or_default();
    let thinks: Vec<_> = rows("calls")
        .iter()
        .map(|c| c["think_ms"].clone())
        .collect();
    let timed =
        |row: &serde_json::Value| row.get("act_ms").is_some() && row.get("settle_ms").is_some();
    let steps: Vec<bool> = rows("steps").iter().map(timed).collect();
    assert_eq!(thinks, vec![200; 3], "the thinking time of each call");
    assert_eq!(steps, vec![true; 3], "the acting and settling of each step");
}

#[then(expr = "実行記録に判断モデルの呼び出し回数、トークン数、算出したコストが含まれる")]
async fn usage(world: &mut QaWorld) {
    world.run().await;
    let report = found(world.report(), "the report was written");
    let expected =
        serde_json::json!({"calls": 3, "input_tokens": 0, "output_tokens": 0, "cost": 0.0});
    assert_eq!(report["usage"], expected, "the usage");
}
