//! Steps of the Rule 終了の判定: the first scenarios and the stuck count. The
//! scenarios whose steps are all here are named in `READY`.

use cucumber::{given, then, when};
use webqa_core::machine::ExitReason;

use crate::ending::{no_progress, revealed, stuck_screen, written_reasons};
use crate::judge::Reply;
use crate::page::PageSpec;
use crate::shared::{login_only, url_done};
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "done_when を満たしたら、判断モデルに問い合わせずにゴール達成で終了する",
    "done_when を満たさないまま判断モデルが done を返したら、未検証の終了として記録する",
    "判断モデルが stuck を返したら行き詰まりで終了する",
    "探索済みの要素が増えず、未操作の要素も新しく見えない操作が3回続けば、次のステップの前に行き詰まりで終了する",
    "新しい要素を操作すると行き詰まりの数え直しになる",
    "スクロールで未操作の要素が新しく見えたら、行き詰まりの数え直しになる",
    "スクロールしても未操作の要素が新しく見えなければ、行き詰まりを数え続ける",
    "探索済みの要素が増えず、未操作の要素も新しく見えない操作が閾値の1回手前までなら終了しない",
];

#[given(expr = "現在の段階の done_when 条件が満たされている")]
fn done_when_met(world: &mut QaWorld) {
    world.scene.stages = login_only(url_done("/"));
    world.scene.start = PageSpec::at("/");
}

#[given(expr = "現在の段階の done_when 条件は満たされていない")]
fn done_when_unmet(world: &mut QaWorld) {
    world.scene.stages = login_only(url_done("/home"));
    world.scene.start = PageSpec::at("/").with(PageSpec::button("保存"));
}

#[given(expr = "判断モデルが操作 {string} とゴール達成ありを返す")]
fn replies_with_goal(world: &mut QaWorld, op: String) {
    world.scene.replies.push_back(Reply::op(op, 0.9).goal(1.0));
}

#[then(expr = "実行は未検証の終了として記録されている")]
async fn recorded_unverified(world: &mut QaWorld) {
    world.run().await;
    let lines = world.lines();
    let last = lines.last().and_then(|line| line.get("reason"));
    assert_eq!(world.reason(), Some(ExitReason::ReportedDone), "the reason");
    assert_eq!(last, Some(&"reported_done".into()), "the last line says so");
}

#[then(expr = "実行はゴール達成として記録されていない")]
async fn not_recorded_as_goal(world: &mut QaWorld) {
    world.run().await;
    let reason = world.reason();
    assert_ne!(reason, Some(ExitReason::GoalAchieved), "the reason");
    let goal = written_reasons(world).contains(&"goal_achieved".to_owned());
    assert!(!goal, "no line says the goal was achieved");
}

#[given(expr = "探索済みの要素は {string} だけである")]
fn explored_only(world: &mut QaWorld, name: String) {
    assert_eq!(name, "保存", "the screen has operated 保存");
    stuck_screen(world);
    drop(name);
}

#[given(expr = "判断モデルは毎回、操作 {string} を返し、そのたびに画面は変化する")]
fn scrolls_every_time(world: &mut QaWorld, op: String) {
    assert_eq!(op, "scroll_down", "the scrolls are of no_progress");
    no_progress(world, 3);
    drop(op);
}

#[given(expr = "スクロールしても、まだ表示領域に現れていなかった未操作の要素は現れない")]
const fn nothing_appears(_world: &mut QaWorld) {}

#[when(expr = "{int} ステップを実行し、{int} ステップ目に進もうとする")]
async fn runs_then_tries(world: &mut QaWorld, done: u32, next: u32) {
    assert_eq!(next, done + 1, "the step after the last one");
    world.run().await;
}

#[given(expr = "探索済みの要素が増えず、未操作の要素も新しく見えない操作が2回続いている")]
fn two_without_progress(world: &mut QaWorld) {
    stuck_screen(world);
    no_progress(world, 2);
}

#[given(expr = "探索済みの要素が増えず、未操作の要素も新しく見えない操作が {int} 回続いている")]
fn times_without_progress(world: &mut QaWorld, times: usize) {
    stuck_screen(world);
    no_progress(world, times);
}

#[given(expr = "判断モデルが操作 {string} と未操作の要素 {string} を返す")]
fn operates_new_element(world: &mut QaWorld, op: String, name: String) {
    world.scene.replies.push_back(Reply::op(op, 0.9).on(name));
    let last = world.scene.after.back().unwrap_or(&world.scene.start);
    world.scene.after.push_back(last.clone());
}

#[then(
    expr = "その後、探索済みの要素が増えず、未操作の要素も新しく見えない操作が2回続いても実行は終了しない"
)]
async fn does_not_end_after_two_more(world: &mut QaWorld) {
    no_progress(world, 2);
    world.ran = None;
    world.run().await;
    assert_eq!(world.reason(), None, "the run has not ended");
}

#[given(
    expr = "スクロールすると、まだ表示領域に現れていなかった未操作のリンク {string} が表示領域に現れる"
)]
fn scroll_reveals(world: &mut QaWorld, name: String) {
    assert_eq!(name, "通知設定", "the link revealed is the one of revealed");
    world.scene.after.push_back(revealed());
    drop(name);
}

#[given(expr = "判断モデルが操作 {string} を返し、スクロールで画面は変化する")]
fn scrolls_once(world: &mut QaWorld, op: String) {
    assert_eq!(op, "scroll_down", "the scroll is of no_progress");
    no_progress(world, 1);
    drop(op);
}

#[given(
    expr = "スクロールして表示領域に現れた要素は、どれも探索済みか、この画面ですでに表示領域に現れたことがある"
)]
const fn nothing_new_appears(_world: &mut QaWorld) {}

#[when(expr = "次のステップに進もうとする")]
#[when(expr = "最初のステップに進む")]
async fn tries_next(world: &mut QaWorld) {
    world.run().await;
}

#[then(expr = "実行は行き詰まりとして終了しておらず、判断モデルに問い合わせる")]
async fn asks_instead_of_stuck(world: &mut QaWorld) {
    world.run().await;
    assert_eq!(world.reason(), None, "the run has not ended");
    assert!(world.ran().judge.stalled, "the judge was asked");
}
