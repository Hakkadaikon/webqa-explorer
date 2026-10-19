//! Steps of one Rule. The scenarios whose steps are all here are named in
//! `READY`.

use cucumber::then;
use webqa_core::machine::ExitReason;

use crate::judge::Reply;
use crate::stages_given::index_of;
use crate::world::{Beat, QaWorld};

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "最後でない段階の done_when を満たしたら、判断モデルに問い合わせずに次の段階へ進む",
    "最後の段階の done_when を満たしたら、ゴール達成で終了する",
    "後の段階の done_when だけを満たしても、段階を飛ばさずゴール達成にもしない",
    "実行を開始した時点で最初の段階の done_when を満たしていれば、操作せずに段階を進める",
    "段階を進めた時点で次の段階の done_when も満たしていれば、操作せずにさらに進む",
    "段階を進めたら、固まりと行き詰まりの回数を0に戻す",
    "段階を進めても、判断予算・時間窓の操作回数・探索の記憶は引き継ぐ",
    "done_when に宣言が複数あれば、すべて成り立ったときだけ満たしたとみなす",
    "done_when の要素とテキストは、表示領域の外にあっても成り立つ",
];

/// The names of the stages the log says the scenario moved on to.
fn advanced_names(world: &QaWorld) -> Vec<String> {
    let lines = world.lines();
    let moved = lines
        .iter()
        .filter(|line| line.get("kind").and_then(|k| k.as_str()) == Some("stage_advanced"));
    let names = moved.filter_map(|line| line.get("name").and_then(|n| n.as_str()));
    names.map(str::to_owned).collect()
}

#[then(expr = "段階 {string} に進んでいる")]
async fn advanced(world: &mut QaWorld, name: String) {
    world.run().await;
    assert_eq!(world.state().stage(), index_of(world, &name), "the stage");
    assert!(advanced_names(world).contains(&name), "the log says so");
}

#[then(expr = "段階 {string} のままで、判断モデルに問い合わせている")]
async fn stays_and_asks(world: &mut QaWorld, name: String) {
    world.run().await;
    assert_eq!(world.state().stage(), index_of(world, &name), "the stage");
    assert!(world.when_asked(), "the step asked the judge");
}

#[then(expr = "段階 {string} に進み、実行は終了していない")]
async fn advanced_not_ended(world: &mut QaWorld, name: String) {
    world.run().await;
    assert_eq!(world.state().stage(), index_of(world, &name), "the stage");
    assert_eq!(world.reason(), None, "the run has not ended");
}

#[then(expr = "ブラウザ操作も判断モデルへの問い合わせも、まだ起きていない")]
async fn nothing_yet(world: &mut QaWorld) {
    world.run().await;
    let first = world.when_beat();
    assert!(
        [Beat::Advance, Beat::End].contains(&first),
        "first: {first:?}"
    );
    assert_eq!(world.touched(), vec![], "no operation");
}

#[then(expr = "段階 {string} で操作せずに、実行はゴール達成として終了している")]
async fn goal_without_operating(world: &mut QaWorld, name: String) {
    world.run().await;
    assert_eq!(world.reason(), Some(ExitReason::GoalAchieved), "the reason");
    assert_eq!(world.state().stage(), index_of(world, &name), "the stage");
    assert_eq!(world.touched(), vec![], "no operation");
}

#[then(
    expr = "段階 {string} の done_when を満たしてから、ブラウザ操作も判断モデルへの問い合わせも起きていない"
)]
async fn quiet_after_done(world: &mut QaWorld, name: String) {
    drop(name);
    world.run().await;
    assert_eq!(world.asked(), 0, "the judge was not asked");
    assert_eq!(world.touched(), vec![], "no operation");
}

#[then(
    expr = "その後、画面が変化せず探索済みの要素も増えないステップが {int} 回続いても、ページを開き直さない"
)]
async fn no_reload_after_stage(world: &mut QaWorld, times: usize) {
    let waits = (0..times).map(|_| Reply::op("wait", 0.9));
    world.scene.replies.extend(waits);
    world.ran = None;
    world.run().await;
    let reloads = world.touched().into_iter().filter(|(op, _)| op == "reload");
    assert_eq!(reloads.count(), 0, "the page was not reopened");
}

#[then(expr = "行き詰まりとしても終了しない")]
async fn not_stuck(world: &mut QaWorld) {
    world.run().await;
    let stuck = world.reason().is_some_and(ExitReason::is_stuck);
    assert!(!stuck, "the run did not end as stuck");
}

#[then(expr = "判断モデルの呼び出し回数は {int} 回のままである")]
async fn asked_before_advance(world: &mut QaWorld, times: usize) {
    world.run().await;
    let kinds = world.kinds();
    let before = kinds.iter().take_while(|kind| *kind != "stage_advanced");
    assert_eq!(before.filter(|k| *k == "asked").count(), times, "asked");
}

#[then(expr = "探索済みの要素は {string} のままである")]
async fn explored_stays(world: &mut QaWorld, name: String) {
    world.run().await;
    assert_eq!(world.explored(), [name].into(), "the explored elements");
}

#[then(
    expr = "段階 {string} で判断モデルに1回問い合わせたあと、次のステップで判断予算切れとして終了する"
)]
async fn budget_runs_out(world: &mut QaWorld, name: String) {
    drop(name);
    world.scene.replies.push_back(Reply::op("back", 0.5));
    world.ran = None;
    world.run().await;
    assert_eq!(
        world.reason(),
        Some(ExitReason::BudgetExceeded),
        "the reason"
    );
    assert_eq!(world.asked(), 4, "the questions");
    let kinds = world.kinds();
    let last_ask = kinds.iter().rposition(|kind| kind == "asked");
    let advance = kinds.iter().position(|kind| kind == "stage_advanced");
    assert!(
        last_ask > advance,
        "the last question came after the advance"
    );
}
