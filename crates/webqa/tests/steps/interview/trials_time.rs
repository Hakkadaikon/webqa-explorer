//! Steps of the trials on the time limit: what it cuts between the trials,
//! and while the reset command runs.

use std::time::Duration;

use cucumber::{given, then, when};
use webqa_core::ports::Clock;

use super::seen::TIME_UP;
use super::{End, HOUR, course, ending_of, fill, first, found, has, minutes, think};
use crate::world::InterviewWorld;

#[given(expr = "制限時間は実行の開始から {int} 分に設定されている")]
const fn limit_set(world: &mut InterviewWorld, limit: u64) {
    world.scene.max_duration = Some(minutes(limit));
}

#[given(expr = "1回目の試行が終わり、2回目の試行を始めるために時間窓が明けるのを待っている")]
#[given(expr = "1回目の試行が終わり、2回目の試行を始めるには時間窓が明けるのを待つ必要がある")]
fn waiting_for_window(world: &mut InterviewWorld) {
    let scene = &mut world.scene;
    scene.requests_per_hour = 10;
    // The first page and four operations leave less than half the cap.
    first(scene, 4, End::Goal);
    fill(scene);
}

#[when(expr = "利用者が実行を中断する")]
async fn user_interrupts(world: &mut InterviewWorld) {
    world.scene.stall_sleep = true;
    world.run().await;
}

#[then(expr = "窓が明けるのを待たずに終わっている")]
async fn ended_before_window(world: &mut InterviewWorld) {
    world.run().await;
    let ran = world.ran();
    assert_eq!(ran.clock.elapsed(), Duration::ZERO, "the clock stayed");
    assert_eq!(ran.clock.sleeps, [(HOUR, 0)], "the wait began, never ended");
}

#[then(expr = "台帳は書かれていて、試行の回数は {int} 回として数えられている")]
async fn ledger_counts(world: &mut InterviewWorld, n: usize) {
    world.run().await;
    let finished = found(world.ledger(), "the ledger was written")
        .head
        .finished;
    assert_eq!(finished, n, "the trials the ledger counts");
}

#[when(expr = "実行の開始から {int} 分が経つ")]
async fn time_passes(world: &mut InterviewWorld, limit: u64) {
    assert_eq!(world.scene.max_duration, Some(minutes(limit)), "the limit");
    world.run().await;
}

#[when(expr = "次の窓が明けるのは、実行の開始から {int} 分より後である")]
async fn window_opens_after(world: &mut InterviewWorld, limit: u64) {
    assert_eq!(world.scene.max_duration, Some(minutes(limit)), "the limit");
    assert!(HOUR > minutes(limit), "the next window opens an hour in");
    world.run().await;
}

#[then(expr = "窓が明けるのを待たずに、実行は時間切れとして終わっている")]
async fn timed_out_before_window(world: &mut InterviewWorld) {
    world.run().await;
    let ran = world.ran();
    assert_eq!(ran.ending, Some(TIME_UP), "how the run ended");
    // The limit comes before the end of the first window: it never slept.
    assert_eq!(ran.clock.sleeps, [], "it did not wait");
    assert_eq!(ran.clock.elapsed(), Duration::ZERO, "the clock stayed");
}

#[when(expr = "2回目の試行を続ける")]
async fn second_goes_on(world: &mut InterviewWorld) {
    let scene = &mut world.scene;
    let call = course(scene, 1, End::Open);
    think(scene, call, minutes(30));
    world.run().await;
}

#[then(expr = "2回目の試行は、実行の開始から {int} 分の時点で時間切れとして終わる")]
async fn second_times_out(world: &mut InterviewWorld, at: u64) {
    world.run().await;
    let ended = ending_of(world, 2);
    assert_eq!(ended.as_deref(), Some("time_up"), "how the trial ended");
    // The call that took the clock past the limit was the last thing it did.
    let ended = world.ran().clock.elapsed();
    assert!(ended >= minutes(at), "the limit had passed");
    assert_eq!(ended, minutes(40) + minutes(30), "and nothing came after");
}

#[then(expr = "3回目の試行は始まっていない")]
async fn third_not_started(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(world.ran().launched, 2, "two browsers");
    assert!(!has(world, "trial_started", 3), "no third trial began");
}

#[given(
    expr = "2回目の試行の前の初期化のコマンドが、実行の開始から {int} 分に始まり、{int} 分に正常に終わる"
)]
fn reset_ends_late(world: &mut InterviewWorld, begins: u64, ends: u64) {
    let scene = &mut world.scene;
    let call = first(scene, 0, End::Done);
    think(scene, call, minutes(begins));
    scene.reset_by_run(&[Duration::ZERO, minutes(ends)], None, 300);
    fill(scene);
}

#[then(expr = "初期化のコマンドは途中で止められず、終わるまで待たれている")]
async fn reset_waited_for(world: &mut InterviewWorld) {
    world.run().await;
    // The command moves the clock as it ends.
    let ended = world.ran().clock.elapsed();
    assert_eq!(ended, minutes(61), "it ended on its own");
}

#[then(expr = "2回目の試行のブラウザは開かれておらず、実行は時間切れとして終わっている")]
async fn no_browser_and_time_up(world: &mut InterviewWorld) {
    world.run().await;
    let ran = world.ran();
    assert_eq!(
        (ran.launched, ran.ending),
        (1, Some(TIME_UP)),
        "browsers, end"
    );
}

#[given(
    expr = "制限時間は実行の開始から {int} 分、初期化のコマンドの待ち時間は {int} 分に設定されている"
)]
fn limit_and_timeout(world: &mut InterviewWorld, limit: u64, timeout: u64) {
    world.scene.max_duration = Some(minutes(limit));
    world.scene.reset_with("runs.sh", timeout * 60);
}

#[given(expr = "2回目の試行の前の初期化のコマンドが、実行の開始から {int} 分に始まり、終わらない")]
fn reset_hangs_late(world: &mut InterviewWorld, begins: u64) {
    let scene = &mut world.scene;
    let reset = found(scene.reset.as_ref(), "the timeout is set");
    let timeout = reset.timeout_secs.get();
    let call = first(scene, 0, End::Done);
    think(scene, call, minutes(begins));
    scene.reset_by_run(&[], Some(2), timeout);
    fill(scene);
}

#[when(expr = "実行の開始から {int} 分が経ち、さらに初期化のコマンドが始まってから {int} 分が経つ")]
async fn time_passes_and_command_times_out(world: &mut InterviewWorld, limit: u64, timeout: u64) {
    let reset = found(world.scene.reset.as_ref(), "the command is written");
    assert_eq!(reset.timeout_secs.get(), timeout * 60, "the timeout");
    assert_eq!(world.scene.max_duration, Some(minutes(limit)), "the limit");
    world.run().await;
}

#[then(expr = "初期化のコマンドは止められている")]
async fn reset_stopped(world: &mut InterviewWorld) {
    world.run().await;
    let scene = &world.scene;
    let begun = scene.thinks.front().copied().unwrap_or_default();
    let reset = found(scene.reset.as_ref(), "the command is written");
    let deadline = begun + Duration::from_secs(reset.timeout_secs.get());
    let sleeps = &world.ran().clock.sleeps;
    assert!(
        sleeps.contains(&(deadline, 1)),
        "it waited until the timeout"
    );
    assert_eq!(world.ran().clock.elapsed(), deadline, "and stopped there");
}

#[then(expr = "実行は時間切れとして終わり、初期化のコマンドの失敗としては記録されていない")]
async fn time_up_not_reset_failure(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(world.ran().ending, Some(TIME_UP), "how the run ended");
}

#[then(expr = "終了コードは {int} ではない")]
async fn code_is_not(world: &mut InterviewWorld, code: u8) {
    world.run().await;
    assert_ne!(world.ran().code, code, "the exit code");
}
