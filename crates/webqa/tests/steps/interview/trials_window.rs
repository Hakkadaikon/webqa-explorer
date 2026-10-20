//! Steps of the trials on the window of the run: what the loop counts in it,
//! and when it waits for it before a trial.

use std::time::Duration;

use cucumber::{given, then, when};

use super::{End, HOUR, WINDOW_MS, before_wait, fill, first, launches, second_and_rest};
use super::{started_at, think};
use crate::world::InterviewWorld;

/// The caps the scenes that do not name one run with, by the accesses used
/// after the first trial.
const CAPS: [(usize, u32); 2] = [(3, 8), (10, 10)];

/// Whether a window with `used` of `cap` accesses used makes the loop wait
/// before a trial: what is left, less the first page, is below half the cap.
const fn waits(cap: u32, used: u32) -> bool {
    (cap - used - 1) * 2 < cap
}

#[given(expr = "1時間窓あたりの対象サーバーへのアクセスの上限は {int} 回に設定されている")]
const fn window_cap(world: &mut InterviewWorld, cap: u32) {
    world.scene.requests_per_hour = cap;
}

#[given(expr = "1回目の試行が終わった時点で、実行全体の今の時間窓のアクセスは {int} 回である")]
fn used_after_first(world: &mut InterviewWorld, used: usize) {
    let scene = &mut world.scene;
    let named = CAPS.iter().find(|(count, _)| *count == used);
    scene.requests_per_hour = named.map_or(scene.requests_per_hour, |&(_, cap)| cap);
    // The first page counts; a window that has turned over by the end counts nothing.
    match used {
        0 => {
            let call = first(scene, 0, End::Done);
            think(scene, call, HOUR);
        }
        _ => {
            first(scene, used - 1, End::Goal);
        }
    }
}

#[when(expr = "2回目の試行で、認証状態を入れたあとに最初のページを表示する")]
async fn first_page_shown(world: &mut InterviewWorld) {
    world.scene.auth = true;
    second_and_rest(&mut world.scene, 0);
    world.run().await;
}

#[then(expr = "実行全体の今の時間窓のアクセスは {int} 回になっている")]
async fn used_now(world: &mut InterviewWorld, used: u32) {
    world.run().await;
    let waited = waits(world.scene.requests_per_hour, used);
    let sleeps: Vec<_> = waited.then_some((HOUR, 0)).into_iter().collect();
    let slept = &world.ran().clock.sleeps;
    assert_eq!(slept, &sleeps, "the wait after the second trial");
    let third = u64::from(waited) * WINDOW_MS;
    let starts = (started_at(world, 2), started_at(world, 3));
    assert_eq!(starts, (Some(0), Some(third)), "when each trial began");
}

#[when(expr = "2回目の試行を始める")]
#[when(expr = "2回目の試行を始めようとする")]
async fn second_starts(world: &mut InterviewWorld) {
    // Past any cap of the scenes, so that the trial meets its cap.
    second_and_rest(&mut world.scene, 10);
    world.run().await;
}

/// `time` in milliseconds.
fn elapsed_ms(time: Duration) -> u64 {
    u64::try_from(time.as_millis()).unwrap_or(u64::MAX)
}

#[then(expr = "2回目の試行は待たずに始まっている")]
async fn started_without_waiting(world: &mut InterviewWorld) {
    world.run().await;
    let start = started_at(world, 2).unwrap_or_default();
    let sleeps = &world.ran().clock.sleeps;
    let before = sleeps.iter().filter(|(at, _)| elapsed_ms(*at) <= start);
    assert_eq!(before.count(), 0, "no wait before it");
}

#[then(expr = "2回目の試行の中核の時間窓の上限は {int} 回である")]
#[then(expr = "窓が明けたあとに、中核の時間窓の上限を {int} 回にして2回目の試行が始まる")]
async fn core_cap_is(world: &mut InterviewWorld, cap: usize) {
    world.run().await;
    let made = before_wait(world, "executed", 2);
    assert_eq!(made, cap, "the operations it made");
}

#[then(expr = "2回目の試行は、今の時間窓が明けるまで始まっていない")]
async fn started_after_window(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(started_at(world, 2), Some(WINDOW_MS), "when it began");
    let first = world.ran().clock.sleeps.first().copied();
    assert_eq!(first, Some((HOUR, 0)), "the first thing the run waited for");
}

#[then(expr = "窓が明けるまで、初期化のコマンドは実行されていない")]
async fn reset_waited_for_window(world: &mut InterviewWorld) {
    world.run().await;
    let sleeps = &world.ran().clock.sleeps;
    let window = sleeps.iter().find(|(at, _)| *at == HOUR).copied();
    assert_eq!(window, Some((HOUR, 1)), "one run, then the wait");
}

#[then(expr = "窓が明けたあとに、初期化のコマンドを実行してから2回目の試行が始まる")]
async fn reset_after_window(world: &mut InterviewWorld) {
    world.run().await;
    let second = launches(world).get(1).copied();
    assert_eq!(second, Some((2, 2, 2)), "the run ended before the browser");
    let began = started_at(world, 2);
    assert_eq!(began, Some(WINDOW_MS), "when the second trial began");
}

#[given(expr = "設定の初期化のコマンドが、対象サーバーへ {int} 回リクエストを送る")]
fn reset_requests(world: &mut InterviewWorld, requests: u32) {
    let scene = &mut world.scene;
    // Counted, the requests would leave less than half the cap.
    scene.requests_per_hour = requests * 2;
    scene.reset_by_run(&[], None, 5);
    fill(scene);
}

#[given(expr = "実行全体の今の時間窓のアクセスは {int} 回である")]
fn window_used_is(_world: &mut InterviewWorld, used: u32) {
    assert_eq!(used, 0, "nothing was used yet");
}

#[when(expr = "1回目の試行の前に初期化のコマンドを実行する")]
async fn reset_before_first(world: &mut InterviewWorld) {
    world.run().await;
}

#[then(expr = "実行全体の今の時間窓のアクセスは {int} 回のままである")]
async fn window_used_stays(world: &mut InterviewWorld, used: u32) {
    world.run().await;
    let sleeps = &world.ran().clock.sleeps;
    let waited = sleeps.iter().filter(|(at, _)| *at == HOUR).count();
    let expected = usize::from(waits(world.scene.requests_per_hour, used));
    assert_eq!(waited, expected, "waits for the window");
    assert_eq!(
        started_at(world, 1),
        Some(0),
        "the first trial began at once"
    );
}
