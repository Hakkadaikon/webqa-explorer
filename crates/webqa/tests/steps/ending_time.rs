//! Steps of the Rule 終了の判定: the time limit. The scenarios whose steps
//! are all here are named in `READY`.

use std::time::Duration;

use cucumber::{given, then};
use webqa_core::config::raw::{DoneWhen, Stage};
use webqa_core::machine::ExitReason;
use webqa_core::ports::Clock;

use crate::ending::{explore, settle_time};
use crate::page::PageSpec;
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "探索モードで制限時間に達したら、判断モデルに問い合わせずに時間切れで終了する",
    "制限時間の手前なら、時間切れにせず問い合わせる",
    "決め打ちシナリオでも、実行時間の上限に達したら時間切れで終了する",
    "判断予算切れと時間切れが重なったら、どちらか1つの終了理由で終了する",
    "時間切れとゴール達成や行き詰まりが重なったら、ゴール達成や行き詰まりを採る",
    "時間窓が明けるのを待っている間に制限時間に達したら、窓を待たずに時間切れで終了する",
];

#[given(expr = "探索モードを制限時間 {int} 秒で実行している")]
fn explores_for(world: &mut QaWorld, secs: u64) {
    explore(&mut world.scene);
    world.scene.max_duration = Some(Duration::from_secs(secs));
}

#[given(expr = "未操作の要素が残っていて、行き詰まりは検出されておらず、判断予算も残っている")]
const fn something_left(_world: &mut QaWorld) {}

#[given(expr = "実行の開始から {int} 秒が経った")]
const fn started_ago(world: &mut QaWorld, secs: u64) {
    world.scene.clock_start = Duration::from_secs(secs);
}

#[given(expr = "決め打ちシナリオを実行時間の上限 {int} 秒で実行している")]
fn scenario_for(world: &mut QaWorld, secs: u64) {
    let done_when = DoneWhen {
        url: Some("/home".to_owned()),
        element: None,
        text: None,
    };
    world.scene.stages = vec![Stage {
        name: "ログイン".to_owned(),
        goal: "ログインする".to_owned(),
        done_when,
    }];
    world.scene.max_duration = Some(Duration::from_secs(secs));
    world.scene.start.elements = vec![PageSpec::button("保存")];
}

#[given(expr = "制限時間に達している")]
fn time_is_up(world: &mut QaWorld) {
    world.scene.marks.insert("time_up", 1);
    settle_time(world);
}

#[then(expr = "実行は判断予算切れか時間切れのどちらか1つとして終了している")]
async fn ended_by_budget_or_time(world: &mut QaWorld) {
    world.run().await;
    let reason = world.reason();
    let either = [
        Some(ExitReason::BudgetExceeded),
        Some(ExitReason::TimeExceeded),
    ];
    assert!(either.contains(&reason), "one of the two: {reason:?}");
}

#[given(expr = "時間窓が明けるのは、制限時間に達するより後である")]
const fn window_opens_later(_world: &mut QaWorld) {}

#[then(expr = "制限時間に達した時点で、実行は時間切れとして終了している")]
async fn timed_out_at_the_limit(world: &mut QaWorld) {
    world.run().await;
    let ended = (world.reason(), world.ran().clock.elapsed());
    let expected = (Some(ExitReason::TimeExceeded), Duration::from_secs(60));
    assert_eq!(ended, expected, "it ended at the limit");
}

#[then(expr = "時間窓が明けるまで待っていない")]
async fn did_not_wait_for_the_window(world: &mut QaWorld) {
    world.run().await;
    assert_eq!(
        world.sleeps(),
        [Duration::from_secs(60)],
        "it slept to the limit only"
    );
}
