//! Steps of the Rule of the user's interrupt. The scenarios whose steps are
//! all here are named in `READY`.

use std::time::Duration;

use cucumber::{given, then, when};
use serde_json::Value;
use webqa_core::machine::ExitReason;
use webqa_core::ports::Clock;

use crate::judge::Reply;
use crate::scene::{Stall, found};
use crate::store::Hold;
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "待っている間に中断したら、記憶・終了のイベント・実行結果を書いてから中断で終了する",
    "ブラウザ操作の途中で中断したら、ページの応答を待たずに記録を書いて中断で終了する",
    "中断で終了した実行は、それ以上状態を変えない",
    "ほかの理由で終了したあとに中断しても、終了理由は変わらない",
    "中断で終わったときの終了コードは、確定の問題の有無で0か1になる",
    "記録を書いている間に2回目の中断を受けたら、書き終わりを待たずに終了コード3で終わる",
];

/// The page's response time of the base.
const PAGE_TIMEOUT: Duration = Duration::from_secs(10);

/// Where the interrupt fires, from 1. 0 is no call: the gate fires it.
fn fire_at(world: &mut QaWorld, call: u64) {
    world.scene.marks.insert("fire", call);
}

#[given(expr = "3 ステップを実行し、探索済みの要素は {string} である")]
fn explored_three(world: &mut QaWorld, name: String) {
    let buttons = [name.as_str(), "ヘルプ", "設定", "検索"];
    for shown in buttons {
        world.scene.show_button(shown);
    }
    for clicked in buttons.iter().take(3) {
        let click = Reply::op("click", 0.9).on(*clicked);
        world.scene.replies.push_back(click);
    }
    drop(name);
}

#[given(expr = "次のステップの開始を待っている")]
fn waits_for_step(world: &mut QaWorld) {
    fire_at(world, 10);
}

#[given(expr = "判断モデルの応答を待っている")]
fn waits_for_judge(world: &mut QaWorld) {
    fire_at(world, 0);
}

#[given(expr = "時間窓が明けることを待っている")]
fn waits_for_window(world: &mut QaWorld) {
    world.scene.limits.insert("requests_per_hour", 3);
    world.scene.stall = Stall::Sleep;
    fire_at(world, 0);
}

#[when(expr = "利用者が実行を中断する")]
async fn user_interrupts(world: &mut QaWorld) {
    let call = world.scene.marks.get("fire").copied().unwrap_or(1);
    world.scene.user_interrupts = true;
    world
        .scene
        .fire_on
        .insert(usize::try_from(call).unwrap_or(0));
    world.run().await;
}

#[then(expr = "保存した探索の記憶に {string} が含まれる")]
async fn memory_has(world: &mut QaWorld, name: String) {
    world.run().await;
    assert!(world.explored().contains(&name), "{name} is kept");
}

#[then(expr = "イベントログに終了のイベントが書かれている")]
async fn log_ends(world: &mut QaWorld) {
    world.run().await;
    let kinds = world.kinds();
    let last = kinds.last().map(String::as_str);
    assert_eq!(last, Some("run_finished"), "the log ends");
}

#[then(expr = "実行結果が出力されている")]
async fn report_written(world: &mut QaWorld) {
    world.run().await;
    assert!(world.report().is_some(), "the report is written");
}

#[given(expr = "判断モデルが操作 {string} と対象要素 {string} を返した")]
fn judge_replied(world: &mut QaWorld, op: String, name: String) {
    world.scene.show_button(name.clone());
    world.scene.replies.push_back(Reply::op(op, 0.9).on(name));
}

#[given(expr = "{string} をクリックしたあと、ページの応答を待っている")]
fn page_pending(world: &mut QaWorld, name: String) {
    world.scene.stall = Stall::Touch;
    fire_at(world, 0);
    drop(name);
}

#[then(
    expr = "ページの応答の待ち時間が過ぎるのを待たずに、実行は利用者による中断として終了している"
)]
async fn aborted_early(world: &mut QaWorld) {
    world.run().await;
    let early = world.ran().clock.elapsed() < PAGE_TIMEOUT;
    let seen = (world.reason(), early);
    assert_eq!(seen, (Some(ExitReason::UserAborted), true), "aborted early");
}

#[then(expr = "探索の記憶、終了のイベント、実行結果が書かれている")]
async fn records_written(world: &mut QaWorld) {
    world.run().await;
    let memory = world
        .ran()
        .store
        .memories
        .contains_key(&world.scene.origin());
    let kinds = world.kinds();
    let last = kinds.last().map(String::as_str);
    let seen = (memory, last, world.report().is_some());
    assert_eq!(seen, (true, Some("run_finished"), true), "the records");
}

#[given(expr = "実行が利用者による中断として終了している")]
fn aborted_by_user(world: &mut QaWorld) {
    world.scene.user_interrupts = true;
    world.scene.fire_on.clear();
}

#[when(expr = "さらに1ステップを実行しようとし、時刻も進む")]
async fn step_after_end(world: &mut QaWorld) {
    world.run().await;
    world.poke(Duration::from_secs(3600));
}

#[then(expr = "終了理由は利用者による中断のままである")]
async fn still_aborted(world: &mut QaWorld) {
    world.run().await;
    let poked = found(world.poked.as_ref(), "a step was poked");
    assert_eq!(poked.reason(), Some(ExitReason::UserAborted), "the reason");
}

#[then(expr = "探索の記憶は書き出したときのままである")]
async fn memory_unchanged(world: &mut QaWorld) {
    world.run().await;
    let origin = world.scene.origin();
    let poked = found(world.poked.as_ref(), "a step was poked");
    let now = poked.state().memory().to_json(&origin);
    let stored = world.ran().store.memories.get(&origin);
    let written: Value =
        serde_json::from_str(found(stored, "the memory was written")).unwrap_or_default();
    assert_eq!(now, written, "the memory is as written");
}

#[given(expr = "実行中に記録した問題は 無い である")]
fn no_issue(world: &mut QaWorld) {
    world.scene.replies.push_back(Reply::op("wait", 0.9));
}

#[given(expr = "実行中に記録した問題は エラー表示の問題1件 である")]
fn error_issue(world: &mut QaWorld) {
    world
        .scene
        .replies
        .push_back(Reply::op("wait", 0.9).error(0.9));
}

#[given(expr = "実行中に記録した問題は 参考情報1件だけ である")]
fn hint_only(world: &mut QaWorld) {
    world.scene.probes = true;
    world
        .scene
        .replies
        .push_back(Reply::op("wait", 0.9).confusing(0.9));
}

#[when(expr = "利用者が実行を中断し、記録を書き終えて終わる")]
async fn interrupt_and_finish(world: &mut QaWorld) {
    world.scene.user_interrupts = true;
    world.run().await;
}

#[given(expr = "利用者が実行を中断し、記録を書いている")]
fn interrupt_while_writing(world: &mut QaWorld) {
    world.scene.user_interrupts = true;
    world.scene.fire_on.clear();
    world.scene.store_hold = Hold::Forever;
}

#[when(expr = "書き終わる前に、利用者がもう一度中断する")]
async fn interrupt_again(world: &mut QaWorld) {
    world.run().await;
}

#[then(expr = "記録を書き終わるのを待たずに終わっている")]
async fn ended_unwritten(world: &mut QaWorld) {
    world.run().await;
    let ran = world.ran();
    let seen = (format!("{:?}", ran.ending), world.report().is_none());
    assert_eq!(seen, ("Some(Abandoned)".to_owned(), true), "abandoned");
}
