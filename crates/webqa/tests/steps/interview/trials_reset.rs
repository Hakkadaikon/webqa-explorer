//! Steps of the trials on when the reset command runs, and what a failure of
//! it does to the run.

use cucumber::{given, then};
use webqa::interview::reset::ResetError;
use webqa::interview::trials::Ending;

use super::{fill, found, has, launches};
use crate::world::InterviewWorld;

#[given(expr = "設定に初期化のコマンドが書かれている")]
fn reset_written(world: &mut InterviewWorld) {
    world.scene.reset_by_run(&[], None, 5);
    fill(&mut world.scene);
}

#[then(expr = "初期化のコマンドは {int} 回実行されている")]
async fn reset_ran(world: &mut InterviewWorld, n: usize) {
    world.run().await;
    let started: Vec<usize> = launches(world).iter().map(|launch| launch.1).collect();
    assert_eq!(started, (1..=n).collect::<Vec<_>>(), "runs at each launch");
}

#[then(expr = "各回の実行は、その試行でブラウザを開く前に終わっている")]
async fn reset_ended_first(world: &mut InterviewWorld) {
    world.run().await;
    let all = launches(world);
    let expected: Vec<_> = (1..=all.len()).map(|n| (n, n, n)).collect();
    assert_eq!(all, expected, "run n ended before browser n opened");
}

#[given(expr = "設定に初期化のコマンドが書かれていない")]
fn reset_not_written(world: &mut InterviewWorld) {
    world.scene.reset = None;
    fill(&mut world.scene);
}

#[then(expr = "試行の前に外部のコマンドは実行されていない")]
async fn no_command(world: &mut InterviewWorld) {
    world.run().await;
    let ran = world.ran();
    let untouched: Vec<_> = launches(world).iter().map(|l| (l.1, l.2)).collect();
    assert_eq!(
        untouched,
        vec![(0, 0); ran.launched],
        "no run before a browser"
    );
    assert_eq!(ran.clock.sleeps, [], "nothing was waited for");
}

#[given(expr = "2回目の試行の前の初期化のコマンドが 0 以外の終了コードで終わる")]
fn reset_exits(world: &mut InterviewWorld) {
    let scene = &mut world.scene;
    scene.reset_with("counts.sh", 5);
    scene.env.push(("RESET_ALLOW".into(), "1".into()));
    fill(scene);
}

#[given(expr = "2回目の試行の前の初期化のコマンドが 設定の待ち時間内に終わらない")]
fn reset_hangs(world: &mut InterviewWorld) {
    world.scene.reset_by_run(&[], Some(2), 5);
    fill(&mut world.scene);
}

#[then(expr = "2回目の試行は始まっていない")]
async fn second_not_started(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(world.ran().launched, 1, "only the first browser opened");
    assert!(!has(world, "trial_started", 2), "no second trial began");
}

#[then(expr = "初期化のコマンドが失敗したことが記録されている")]
async fn reset_failure_recorded(world: &mut InterviewWorld) {
    world.run().await;
    let error = world
        .scene
        .reset_hangs_from
        .map_or(ResetError::Exit(1), |_| ResetError::TimedOut);
    let ending = world.ran().ending;
    assert_eq!(ending, Some(Ending::ResetFailed(error)), "how it ended");
}

#[then(expr = "1回目の試行までの結果で台帳が書かれたあとに、終了コード {int} で終わっている")]
async fn ledger_then_code(world: &mut InterviewWorld, code: u8) {
    world.run().await;
    let finished = found(world.ledger(), "the ledger was written")
        .head
        .finished;
    assert_eq!((finished, world.ran().code), (1, code), "trials, code");
}
