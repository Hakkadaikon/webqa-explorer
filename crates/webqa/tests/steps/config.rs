//! Steps of the Rule of the configuration and the outside boundaries. The
//! scenarios whose steps are all here are named in `READY`.

use std::time::Duration;

use cucumber::{given, then, when};
use webqa_core::event::Mode;
use webqa_core::machine::ExitReason;
use webqa_core::ports::Clock;
use webqa_core::redact::Rule;

use crate::judge::Reply;
use crate::page::PageSpec;
use crate::scene::found;
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "テスト対象・入力値・段階を YAML の設定で定義して実行する",
    "決め打ちシナリオと探索モードを同じツールで実行する",
    "指定した認証状態を注入してから探索を始める",
    "設定で指定した機密情報は判断モデルに送る前に伏せる",
    "secret を false にした入力の値は、判断モデルに送る画面状態で伏せない",
    "secret を false にした入力の値も、イベントログの操作の記録には書かない",
    "secret を書かない入力の値は、判断モデルに送る画面状態で伏せる",
    "判断モデルの API キーの設定に secret が書かれていたら設定エラーにする",
    "API キーは設定ファイルに書かず、環境変数から読む",
    "TLS 証明書の検証を無効にする設定は受け付けない",
];

/// The second stage, added to the one of the base.
const SECOND_STAGE: &str =
    "    - name: 投稿\n      goal: 投稿する\n      done_when:\n        text: 投稿しました\n";

/// The address pattern of the base.
const EMAIL: &str = r"[\w.+-]+@[\w-]+\.[\w.]+";

#[given(expr = "YAML の設定に対象 URL、入力値、2つの段階とそれぞれの done_when が書かれている")]
fn two_stages(world: &mut QaWorld) {
    let stage = "        url: /home\n";
    world.configure(stage, &format!("{stage}{SECOND_STAGE}"));
    world.scene.show_button("ログイン");
    let click = Reply::op("click", 0.9).on("ログイン");
    world.scene.replies.push_back(click);
    let home = PageSpec::at("/home").with(PageSpec::button("投稿"));
    world.scene.after.push_back(home);
}

#[when(expr = "その設定で決め打ちシナリオを実行する")]
async fn run_scenario(world: &mut QaWorld) {
    world.scene.mode = Mode::Scenario;
    world.run().await;
}

#[then(expr = "1つ目の段階の done_when を満たしたあと、2つ目の段階へ進んでいる")]
async fn second_stage_reached(world: &mut QaWorld) {
    world.run().await;
    let lines = world.lines();
    let advanced = lines
        .iter()
        .position(|line| line["kind"] == "stage_advanced" && line["name"] == "投稿");
    let executed = world.kinds().iter().position(|kind| kind == "executed");
    let after = advanced.zip(executed).map(|(next, first)| next > first);
    assert_eq!(after, Some(true), "advanced after the first stage's step");
    assert_eq!(world.state().stage(), 1, "the second stage");
}

#[when(expr = "同じ設定で決め打ちシナリオ実行と、{int} 秒の探索モードをそれぞれ起動する")]
async fn both_modes(world: &mut QaWorld, secs: u64) {
    world.configured();
    world.scene.mode = Mode::Scenario;
    world.run().await;
    assert_eq!(world.ran().config_error, None, "the scenario starts");
    world.scene.marks.insert("scenario", 1);
    world.ran = None;
    world.scene.mode = Mode::Explore;
    let from = "  error_confidence: 0.8\n";
    world.configure(from, &format!("{from}  max_duration_secs: {secs}\n"));
    // A second button is left untouched, or the exploration is complete
    // before the time is up.
    world.scene.show_button("保存");
    world.scene.show_button("ヘルプ");
    let at = Duration::from_secs(secs);
    let click = Reply::op("click", 0.9).on("保存").at(at);
    world.scene.replies.push_back(click);
    world.run().await;
}

#[then(expr = "どちらも同じツールで実行でき、探索モードは遅くとも {int} 秒の時点で終わる")]
async fn both_ran(world: &mut QaWorld, secs: u64) {
    world.run().await;
    let ran = world.ran();
    let within = ran.clock.elapsed() <= Duration::from_secs(secs);
    let seen = (world.scene.marks.get("scenario"), &ran.config_error, within);
    assert_eq!(seen, (Some(&1), &None, true), "both ran, in time");
    assert_eq!(world.reason(), Some(ExitReason::TimeExceeded), "the reason");
}

#[given(expr = "設定にログイン済みのセッション情報が指定されている")]
fn session_given(world: &mut QaWorld) {
    world.configured();
    world.scene.show_button("ログアウト");
}

#[when(expr = "探索モードを起動する")]
async fn start_explore(world: &mut QaWorld) {
    world.scene.mode = Mode::Explore;
    world.run().await;
}

#[then(expr = "最初の画面状態はログイン済みの画面である")]
async fn logged_in_screen(world: &mut QaWorld) {
    world.run().await;
    let texts = &world.ran().judge.texts;
    let first = found(texts.first(), "the judge was asked");
    let shown = first.lines().any(|line| line.contains("\"ログアウト\""));
    assert_eq!(world.ran().config_error, None, "the configuration is read");
    assert!(shown, "the first screen is the logged-in one: {first}");
}

#[given(expr = "設定で {string} を伏せる対象に指定している")]
fn redacts(world: &mut QaWorld, name: String) {
    world.scene.redact.push(Rule::pattern(&name, EMAIL));
    drop(name);
}

#[given(expr = "対象ページに {string} が表示されている")]
fn page_shows(world: &mut QaWorld, value: String) {
    world.scene.start.elements.push(PageSpec::link(&value));
    drop(value);
}

/// Replaces the input of the base with the one `name` that types `value`,
/// written with `secret`, which is what follows the reference.
fn input_typing(world: &mut QaWorld, name: &str, value: String, secret: &str) {
    let input = format!("{name}: {{ env: TEST_OPEN{secret} }}");
    world.configure("メールアドレス: { env: TEST_EMAIL }", &input);
    world.scene.env.insert("TEST_OPEN".to_owned(), value);
}

#[given(expr = "設定で入力欄 {string} は値 {string} を打ち、伏せない指定にしている")]
fn open_input(world: &mut QaWorld, name: String, value: String) {
    input_typing(world, &name, value, ", secret: false");
    drop(name);
}

#[given(expr = "設定で入力欄 {string} は値 {string} を打ち、secret を書いていない")]
fn unmarked_input(world: &mut QaWorld, name: String, value: String) {
    input_typing(world, &name, value, "");
    drop(name);
}

#[then(expr = "判断モデルに送った画面状態に {string} が含まれる")]
async fn judge_text_shows(world: &mut QaWorld, value: String) {
    world.run().await;
    let texts = &world.ran().judge.texts;
    let shown = texts.iter().filter(|t| t.contains(&value)).count();
    assert_ne!(shown, 0, "the value was hidden: {texts:?}");
    drop(value);
}

#[then(expr = "判断モデルに送った画面状態に {string} は含まれない")]
async fn judge_text_hides(world: &mut QaWorld, value: String) {
    world.run().await;
    let texts = &world.ran().judge.texts;
    let leaked: Vec<&String> = texts.iter().filter(|t| t.contains(&value)).collect();
    assert_eq!(leaked.len(), 0, "the secret was sent: {}", leaked.len());
}

#[given(expr = "判断モデルの API キーが環境変数で与えられ、設定ファイルには参照だけが書かれている")]
fn key_from_env(world: &mut QaWorld) {
    world.configured();
    world.scene.replies.push_back(Reply::op("wait", 0.9));
}

#[then(expr = "判断モデルの呼び出しに成功している")]
async fn judge_succeeded(world: &mut QaWorld) {
    world.run().await;
    let seen = (&world.ran().config_error, world.answered());
    assert_eq!(seen, (&None, 1), "the judge answered once");
}
