//! Steps of the rule on the load on the target server: the time window.

use std::time::Duration;

use cucumber::{given, then, when};
use webqa_core::ports::Clock;

use crate::judge::Reply;
use crate::page::PageSpec;
use crate::scene::found;
use crate::shared::first_wait;
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "1時間窓の操作回数が上限に達したら、窓が明けるまで問い合わせも操作もしない",
    "時間窓は時刻で区切られ、上限に達していなくても満了する",
    "ガードレールで止めて実行しなかった提案は、時間窓の操作回数に数えない",
    "ページの再読み込みや許可ドメインへの戻りも、時間窓の操作回数に数える",
    "時間窓の操作回数が上限に達していたら、再読み込み・戻り・離脱も窓が明けるまで待つ",
];

/// The length of a time window.
const WINDOW: Duration = Duration::from_secs(3600);

#[given(expr = "1時間窓あたりの操作回数の上限を {int} 回に設定している")]
fn set_cap(world: &mut QaWorld, cap: u32) {
    world.scene.limits.insert("requests_per_hour", cap);
}

#[given(expr = "現在の時間窓で {int} 回操作した")]
fn spent(world: &mut QaWorld, times: u32) {
    let waits = (0..times).map(|_| Reply::op("wait", 0.9));
    world.scene.replies.extend(waits);
}

#[when(expr = "時間窓が明ける前に1ステップを実行しようとする")]
async fn try_before_the_window_ends(world: &mut QaWorld) {
    world.run().await;
}

/// The kinds of the log lines from the last `executed` before the first
/// `waited`, which the core spent waiting for the window.
fn before_waiting(world: &QaWorld) -> Vec<String> {
    let kinds = world.kinds();
    let wait = first_wait(&kinds);
    let before = found(kinds.get(..wait), "the lines before the wait");
    let last = found(
        before.iter().rposition(|kind| kind == "executed"),
        "an operation before the wait",
    );
    found(before.get(last..), "the lines from the operation").to_vec()
}

#[then(expr = "判断モデルに問い合わせておらず、ブラウザ操作もしていない")]
async fn idle_until_the_window_ends(world: &mut QaWorld) {
    world.run().await;
    let between = before_waiting(world);
    let busy = ["asked", "decided", "recovered"];
    assert!(
        between.iter().all(|kind| !busy.contains(&kind.as_str())),
        "{between:?}"
    );
    assert_eq!(world.sleeps(), [WINDOW], "it waited for the window");
}

#[then(expr = "時間窓が明けたあとのステップでは問い合わせる")]
async fn asks_after_the_window(world: &mut QaWorld) {
    world.run().await;
    assert_eq!(world.asked(), world.answered() + 1, "one more question");
    assert!(world.ran().judge.stalled, "the question was left waiting");
    assert_eq!(world.ran().clock.elapsed(), WINDOW, "after the window");
}

#[when(expr = "時間窓の終わりの時刻を過ぎる")]
async fn pass_the_window_end(world: &mut QaWorld) {
    // The first operation changes the screen, so the three waits do not end the run as stuck.
    let changed = PageSpec::at("/").with(PageSpec::button("保存"));
    world.scene.after.push_back(changed);
    let late = Duration::from_secs(3601);
    let replies = [Reply::op("wait", 0.9).at(late), Reply::op("wait", 0.9)];
    world.scene.replies.extend(replies);
    world.run().await;
}

#[then(expr = "新しい時間窓では {int} 回まで操作できる")]
async fn new_window_cap(world: &mut QaWorld, cap: usize) {
    world.run().await;
    assert_eq!(
        world.touched().len(),
        1 + cap,
        "the operations of two windows"
    );
    assert_eq!(
        world.sleeps(),
        [WINDOW * 2],
        "it waited for the third window"
    );
}

#[then(expr = "対象サーバへアクセスしていない")]
async fn no_access(world: &mut QaWorld) {
    world.run().await;
    assert!(
        !before_waiting(world).contains(&"recovered".to_owned()),
        "no access"
    );
}

#[given(expr = "判断モデルが操作 {string} と禁止要素 {string} を返す")]
fn proposes_forbidden(world: &mut QaWorld, op: String, name: String) {
    let scene = &mut world.scene;
    scene.start.elements.push(PageSpec::button(&name));
    scene.start.elements.push(PageSpec::button("保存"));
    scene.replies.push_back(Reply::op(op, 0.9).on(&name));
    drop(name);
}
