//! Steps of the Rule 終了の判定: the budget of decisions, and the end that
//! stays. The scenarios whose steps are all here are named in `READY`.

use std::time::Duration;

use cucumber::{given, then, when};
use webqa_core::machine::{ExitReason, StuckKind};

use crate::ending::{apply_goal, held, no_progress, settle_time, stuck_screen};
use crate::judge::Reply;
use crate::page::PageSpec;
use crate::reason::Named;
use crate::scene::Scene;
use crate::shared::{login_only, text_done};
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "判断モデルの呼び出しが上限に達したら実行を止める",
    "終了の条件が重なったら、ゴール達成、行き詰まり、判断予算切れの順に1つだけ採る",
    "行き詰まりと判断予算切れが重なったら行き詰まりで終了する",
    "固まりと判断予算切れが重なったら、ページを開き直さずに判断予算切れで終了する",
    "判断モデルの呼び出しが上限の1回手前なら、まだ問い合わせる",
    "ブラウザ操作が起きなかったステップは固まりにも行き詰まりにも数えず、判断予算切れで止まる",
    "終了した実行はそれ以上ステップを進めない",
];

/// The step that sets how many calls the judge may make.
fn cap(scene: &mut Scene, calls: u32) {
    scene.limits.insert("max_decisions", calls);
}

#[given(expr = "判断モデルの呼び出し上限を {int} 回に設定し、{int} 回呼び出し済みである")]
fn cap_and_called(world: &mut QaWorld, calls: u32, called: u64) {
    cap(&mut world.scene, calls);
    held(world, usize::try_from(called).unwrap_or_default());
    world.scene.marks.insert("called", called);
}

#[given(expr = "判断モデルの呼び出し上限を {int} 回に設定している")]
fn cap_only(world: &mut QaWorld, calls: u32) {
    cap(&mut world.scene, calls);
}

/// Keeps the last `called` answers, and the screens of the operations
/// among them. A step of the common steps adds answers of its own (the
/// freeze of three unchanged screens adds three clicks), which would call
/// the judge more often than the scene says.
fn keep_called(scene: &mut Scene) {
    let Some(&called) = scene.marks.get("called") else {
        return;
    };
    let cut = scene
        .replies
        .len()
        .saturating_sub(usize::try_from(called).unwrap_or_default());
    let dropped = scene.replies.drain(..cut);
    let operated = dropped.filter(|reply| !is_held(reply)).count();
    let skipped = operated.min(scene.after.len());
    scene.after.drain(..skipped);
}

/// Whether the reply stops the step: no browser operation.
fn is_held(reply: &Reply) -> bool {
    match reply {
        Reply::Answer(spoken) => spoken.op.as_ref().is_some_and(|(op, _)| op == "back"),
        Reply::Fail(_) => true,
    }
}

#[given(expr = "done_when 条件は満たされておらず、行き詰まりも検出されていない")]
fn nothing_ends_the_run(world: &mut QaWorld) {
    keep_called(&mut world.scene);
    settle_time(world);
}

#[given(expr = "done_when 条件が満たされている")]
fn done_when_is_met(world: &mut QaWorld) {
    world.scene.stages = login_only(text_done("ようこそ"));
    world.scene.marks.insert("goal", 1);
    apply_goal(world);
}

#[given(expr = "done_when 条件は満たされていない")]
const fn done_when_is_not_met(_world: &mut QaWorld) {}

#[given(expr = "行き詰まりも検出されている")]
#[given(expr = "行き詰まりが検出されている")]
fn stuck_is_detected(world: &mut QaWorld) {
    stuck_screen(world);
    let keep = world.scene.replies.len().saturating_sub(3);
    world.scene.replies.truncate(keep);
    world.scene.after.clear();
    no_progress(world, 3);
    apply_goal(world);
    settle_time(world);
}

#[then(expr = "対象ページを開き直していない")]
async fn not_reloaded(world: &mut QaWorld) {
    world.run().await;
    let reloaded = world.touched().iter().any(|(op, _)| op == "reload");
    assert!(!reloaded, "the page was not reloaded");
}

#[then(expr = "判断モデルへの問い合わせ回数は {int} 回のままである")]
#[then(expr = "判断モデルへの問い合わせ回数は {int} 回になっている")]
async fn asked_exactly(world: &mut QaWorld, times: usize) {
    world.run().await;
    assert_eq!(world.asked(), times, "the judge was asked this often");
}

#[then(expr = "実行は{reason}として終了していない")]
async fn not_ended_as(world: &mut QaWorld, named: Named) {
    world.run().await;
    assert!(
        !named.matches(world.reason()),
        "the run did not end as {named:?}"
    );
}

#[given(expr = "判断モデルは毎回 禁止要素 {string} への click を返す")]
fn forbidden_every_time(world: &mut QaWorld, name: String) {
    let all = PageSpec::button(&name);
    world.scene.start.elements = vec![all, PageSpec::button("保存")];
    every_time(world, move || Reply::op("click", 0.9).on(name.clone()));
}

#[given(expr = "判断モデルは毎回 確信度 {float} の back を返す")]
fn unsure_every_time(world: &mut QaWorld, p: f64) {
    world.scene.show_button("保存");
    every_time(world, || Reply::op("back", p));
}

#[given(expr = "判断モデルは毎回 存在しない要素番号 {int} への click を返す")]
fn missing_every_time(world: &mut QaWorld, number: usize) {
    world.scene.show_button("保存");
    every_time(world, || Reply::op("click", 0.9).on_number(number));
}

/// The same reply as many times as the budget allows.
fn every_time(world: &mut QaWorld, reply: impl Fn() -> Reply) {
    let calls = world
        .scene
        .limits
        .get("max_decisions")
        .copied()
        .unwrap_or_default();
    let all = (0..calls).map(|_| reply());
    world.scene.replies.extend(all);
}

#[then(expr = "対象ページを開き直しておらず、行き詰まりとして終了していない")]
async fn neither_reload_nor_stuck(world: &mut QaWorld) {
    not_reloaded(world).await;
    let stuck = Named(ExitReason::Stuck(StuckKind::NoProgress));
    assert!(
        !stuck.matches(world.reason()),
        "the run did not end as stuck"
    );
}

#[when(expr = "さらに1ステップを実行しようとする")]
async fn one_more_step(world: &mut QaWorld) {
    world.run().await;
    world.poke(Duration::from_secs(1));
}
