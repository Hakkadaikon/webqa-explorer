//! Steps that more than one Rule uses. A Rule's own file does not word
//! these again, or cucumber finds them ambiguous.

use cucumber::{given, then, when};
use webqa_core::event::Mode;
use webqa_core::machine::{ExitReason, Step};

use crate::judge::Reply;
use crate::page::PageSpec;
use crate::reason::Named;
use crate::shared::{login_only, url_done};
use crate::world::QaWorld;

/// No scenario of the feature belongs to these steps alone.
pub const READY: &[&str] = &[];

#[when(expr = "1ステップを実行する")]
#[when(expr = "1ステップを実行し、次のステップに進もうとする")]
async fn one_step(world: &mut QaWorld) {
    world.run().await;
}

#[given(expr = "1時間窓あたりの操作回数の上限を {int} 回に設定し、現在の時間窓で {int} 回操作した")]
fn window_spent(world: &mut QaWorld, cap: u32, spent: u32) {
    world.scene.limits.insert("requests_per_hour", cap);
    let waits = (0..spent).map(|_| Reply::op("wait", 0.9));
    world.scene.replies.extend(waits);
}

#[then(expr = "ブラウザで {string} をクリックしている")]
async fn clicked(world: &mut QaWorld, name: String) {
    world.run().await;
    let touched = world.touched();
    assert!(
        touched.contains(&("click".to_owned(), Some(name))),
        "the browser clicked it: {touched:?}"
    );
}

#[given(expr = "判断モデルが操作 {string} と対象要素 {string} を確信度 {float} で返す")]
fn replies_on(world: &mut QaWorld, op: String, name: String, p: f64) {
    world.scene.replies.push_back(Reply::op(op, p).on(name));
}

#[given(expr = "判断モデルが操作 {string} を返す")]
fn replies(world: &mut QaWorld, op: String) {
    world.scene.replies.push_back(Reply::op(op, 0.9));
}

#[given(expr = "判断モデルにまだ問い合わせていない")]
fn not_asked_yet(world: &mut QaWorld) {
    // `take` because a step takes the world mutably; `is_none` would not.
    assert!(world.ran.take().is_none(), "the run has not started");
    assert!(world.scene.replies.is_empty(), "no reply is queued yet");
}

#[then(expr = "判断モデルに問い合わせている")]
async fn asks(world: &mut QaWorld) {
    world.run().await;
    assert!(world.when_asked(), "the step asked the judge");
}

#[then(expr = "判断モデルへの問い合わせもブラウザ操作も起きない")]
async fn nothing_happens(world: &mut QaWorld) {
    world.run().await;
    let ended = world.poked.as_ref().map(|step| step.reason().is_some());
    assert_eq!(ended, Some(true), "the step finished at once");
}

#[then(expr = "判断モデルへの問い合わせ回数は {int} 回である")]
async fn asked_times(world: &mut QaWorld, times: usize) {
    world.run().await;
    assert_eq!(world.asked(), times, "the judge was asked this often");
}

#[then(expr = "判断モデルへの問い合わせ回数は増えていない")]
async fn not_asked_again(world: &mut QaWorld) {
    world.run().await;
    assert!(!world.when_asked(), "the step did not ask the judge");
}

#[given(expr = "実行がゴール達成として終了している")]
fn goal_reached(world: &mut QaWorld) {
    world.scene.stages = login_only(url_done("/"));
    world.scene.start = PageSpec::at("/");
}

#[then(expr = "実行は{reason}として終了している")]
async fn ended_as(world: &mut QaWorld, named: Named) {
    world.run().await;
    let reason = world.reason();
    assert!(
        named.matches(reason),
        "the run ended as {named:?}: {reason:?}"
    );
}

#[then(expr = "実行は終了していない")]
async fn not_ended(world: &mut QaWorld) {
    world.run().await;
    assert_eq!(world.reason(), None, "the run has not ended");
}

#[then(expr = "実行は終了しておらず、次のステップで判断モデルに問い合わせる")]
async fn waits_for_the_judge(world: &mut QaWorld) {
    world.run().await;
    assert_eq!(world.reason(), None, "the run has not ended");
    assert!(world.ran().judge.stalled, "the judge was left waiting");
}

#[when(expr = "探索モードを起動しようとする")]
async fn start_exploring(world: &mut QaWorld) {
    world.scene.mode = Mode::Explore;
    world.configured();
    world.run().await;
}

/// The start page shows a button named `name`, and the judge clicks it.
fn click_on(world: &mut QaWorld, name: impl Into<String>) {
    let name = name.into();
    world.scene.show_button(name.clone());
    let click = Reply::op("click", 0.9).on(name);
    world.scene.replies.push_back(click);
}

#[given(expr = "探索済みの要素は {string} である")]
fn explored(world: &mut QaWorld, name: String) {
    click_on(world, name);
}

#[given(expr = "操作しても画面が変化しないステップが {int} 回続いている")]
fn unchanged_steps(world: &mut QaWorld, times: usize) {
    for number in 1..=times {
        click_on(world, format!("操作{number}"));
    }
}

#[then(expr = "時間窓の操作回数は {int} 回のままである")]
async fn window_unchanged(world: &mut QaWorld, used: u32) {
    world.run().await;
    assert_eq!(world.window_used(), used, "the window's count");
}

#[then(expr = "終了コードは {int} である")]
async fn exit_code(world: &mut QaWorld, code: u8) {
    world.run().await;
    assert_eq!(world.ran().code, code, "the exit code");
}

#[then(expr = "終了理由はゴール達成のままである")]
async fn still_goal_achieved(world: &mut QaWorld) {
    world.run().await;
    let reason = world
        .poked
        .as_ref()
        .map_or_else(|| world.reason(), Step::reason);
    assert_eq!(reason, Some(ExitReason::GoalAchieved), "the reason stays");
}

#[given(expr = "設定で判断モデル API の TLS 証明書検証を無効にしている")]
fn tls_off(world: &mut QaWorld) {
    world.configure(
        "  timeout_secs: 30\n",
        "  timeout_secs: 30\n  tls_verify: false\n",
    );
}

#[given(expr = "設定ファイルが参照する環境変数が設定されていない")]
fn env_missing(world: &mut QaWorld) {
    world.configured();
    world.scene.env.remove("JEV_API_KEY");
}

#[given(expr = "設定ファイルに判断モデルの API キーの値がそのまま書かれている")]
fn key_written(world: &mut QaWorld) {
    world.configure("api_key: { env: JEV_API_KEY }", "api_key: sk-live-123");
}

#[given(expr = "設定ファイルの判断モデルの API キーの参照に secret が書かれている")]
fn key_marked_secret(world: &mut QaWorld) {
    let marked = "api_key: { env: JEV_API_KEY, secret: false }";
    world.configure("api_key: { env: JEV_API_KEY }", marked);
}

#[then(expr = "起動は設定エラーで失敗する")]
async fn config_fails(world: &mut QaWorld) {
    world.run().await;
    let ran = world.ran();
    assert_ne!(ran.config_error, None, "the configuration was rejected");
    let sentence = ran.config_error.clone().unwrap_or_default();
    let leaked: Vec<&str> = ["sk-live-123", "value-of-"]
        .into_iter()
        .filter(|secret| sentence.contains(secret))
        .collect();
    assert_eq!(leaked, Vec::<&str>::new(), "the sentence hides the secrets");
    assert_eq!(ran.code, 2, "the exit code of a configuration error");
}
