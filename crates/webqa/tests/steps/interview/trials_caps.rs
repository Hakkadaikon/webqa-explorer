//! Steps of the trials on the window of a trial: where it turns over, and
//! the cap it was given.

use cucumber::{given, then, when};
use serde_json::Value;

use super::{End, WINDOW_MS, before_wait, elapsed_of, fill, first, kind_of, minutes};
use super::{second_and_rest, started_at, think, within};
use crate::world::InterviewWorld;

/// The operations of the trial 2, by kind, each as the time they were
/// logged.
fn executed(world: &InterviewWorld) -> Vec<u64> {
    let inside = within(world, 2);
    let lines = inside.iter().filter(|line| kind_of(line) == "executed");
    lines.map(elapsed_of).collect()
}

#[given(expr = "1回目の試行が、実行の開始から {int} 分で終わる")]
fn first_ends_at(world: &mut InterviewWorld, at: u64) {
    let scene = &mut world.scene;
    let call = first(scene, 0, End::Done);
    think(scene, call, minutes(at));
}

#[when(expr = "2回目の試行を実行の開始から {int} 分で始め、実行の開始から {int} 分を過ぎる")]
async fn crosses_the_hour(world: &mut InterviewWorld, start: u64, past: u64) {
    let scene = &mut world.scene;
    scene.requests_per_hour = 6;
    let call = super::plan::course(scene, 5, End::Done);
    // The third decision comes after `past`, the second before it.
    let step = minutes((past - start) / 2 - 1);
    (call..call + 6).for_each(|at| think(scene, at, step));
    fill(scene);
    world.run().await;
}

#[then(expr = "2回目の試行の中核の時間窓は、実行の開始から {int} 分の時点で明けている")]
async fn opened_on_the_hour(world: &mut InterviewWorld, at: u64) {
    world.run().await;
    let boundary = minutes(at).as_millis();
    let (after, before): (Vec<u64>, Vec<u64>) = executed(world)
        .into_iter()
        .partition(|logged| u128::from(*logged) >= boundary);
    assert_eq!(
        (before.len(), after.len()),
        (2, 3),
        "operations before and after"
    );
    assert_eq!(before_wait(world, "executed", 2), 5, "none had to wait");
}

#[then(
    expr = "2回目の試行の中核の時間窓は、2回目を始めてから {int} 分の時点ではなく、その前に明けている"
)]
async fn not_an_hour_from_start(world: &mut InterviewWorld, length: u64) {
    world.run().await;
    let start = started_at(world, 2).unwrap_or_default();
    let over = u128::from(start) + minutes(length).as_millis();
    let early = executed(world)
        .into_iter()
        .filter(|at| u128::from(*at) < over);
    // The cap of the trial is 4: the fifth operation came within an hour of its start.
    assert_eq!(early.count(), 5, "operations within an hour of its start");
}

#[given(expr = "2回目の試行を、中核の時間窓の上限を {int} 回にして始めた")]
fn second_with_cap(world: &mut InterviewWorld, cap: u32) {
    let scene = &mut world.scene;
    let used = scene.requests_per_hour - cap - 1;
    for _ in scene.after.len()..1 {
        first(scene, usize::try_from(used).unwrap_or(1) - 1, End::Goal);
    }
}

#[when(expr = "2回目の試行の途中で、実行全体の時間窓が明ける")]
async fn window_turns(world: &mut InterviewWorld) {
    let scene = &mut world.scene;
    let call = super::plan::course(scene, 7, End::Done);
    think(scene, call, super::HOUR);
    fill(scene);
    world.run().await;
}

#[then(expr = "新しい窓でも、2回目の試行の中核の時間窓の上限は {int} 回のままである")]
async fn cap_kept(world: &mut InterviewWorld, cap: usize) {
    world.run().await;
    assert_eq!(
        before_wait(world, "executed", 2),
        cap,
        "operations before it waited"
    );
    let waited = within(world, 2)
        .into_iter()
        .find(|line| kind_of(line) == "waited");
    let until = waited.and_then(|line| line.get("until_ms").and_then(Value::as_u64));
    assert_eq!(
        until,
        Some(2 * WINDOW_MS),
        "it waited for the window after the new one"
    );
}

#[given(
    expr = "1回目の試行で実行全体の今の窓のアクセスは {int} 回になり、2回目の最初の表示で {int} 回になった"
)]
fn used_by_second_page(world: &mut InterviewWorld, used: usize, after: usize) {
    assert_eq!(after, used + 1, "the first page counts one");
    first(&mut world.scene, used - 1, End::Goal);
}

#[when(expr = "2回目の試行で、今の窓に対象サーバーへ {int} 回アクセスする")]
async fn second_reaches(world: &mut InterviewWorld, accesses: usize) {
    second_and_rest(&mut world.scene, accesses);
    world.run().await;
}

#[then(expr = "2回目の試行は、窓が明けるまで判断モデルへの問い合わせもブラウザ操作もしない")]
async fn second_stands_still(world: &mut InterviewWorld) {
    world.run().await;
    let before = |kind: &str| before_wait(world, kind, 2);
    let done = (before("asked"), before("decided"), before("executed"));
    assert_eq!(done, (5, 5, 5), "what it did before the wait");
    let inside = within(world, 2);
    assert!(
        inside.iter().any(|line| kind_of(line) == "waited"),
        "it waited"
    );
}

#[then(expr = "実行全体の今の窓のアクセスは {int} 回を超えていない")]
async fn window_not_over(world: &mut InterviewWorld, cap: usize) {
    world.run().await;
    let access = |line: &&Value| {
        let kind = kind_of(line);
        (kind == "trial_started" || kind == "executed") && elapsed_of(line) < WINDOW_MS
    };
    let lines = world.lines();
    assert_eq!(
        lines.iter().filter(access).count(),
        cap,
        "accesses of the first window"
    );
}

#[given(expr = "2回目の試行を、時間窓が明けるのを待ってから始めた")]
fn second_after_wait(world: &mut InterviewWorld) {
    first(&mut world.scene, 4, End::Goal);
}

#[when(expr = "2回目の試行の1ステップを実行する")]
async fn second_one_step(world: &mut InterviewWorld) {
    super::plan::course(&mut world.scene, 1, End::Goal);
    fill(&mut world.scene);
    world.run().await;
}

#[then(expr = "2回目の試行の時間窓の操作回数は、そのステップのアクセスだけを数えている")]
async fn counts_its_own(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(executed(world).len(), 1, "the step of the trial");
    assert_eq!(
        started_at(world, 2),
        Some(WINDOW_MS),
        "the trial began after the window"
    );
    assert_eq!(
        world.ran().clock.sleeps,
        [(super::HOUR, 0)],
        "the third trial did not wait"
    );
}
