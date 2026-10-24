//! Steps of the Rule of the exit codes. The scenarios whose steps are all
//! here are named in `READY`.

use cucumber::{given, then, when};

use serde_json::json;
use webqa::run::session::Ending;

use crate::judge::Reply;

use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "設定エラーで起動しなかったら、終了コードは2になる",
    "記録の書き込みに失敗したら、実行を止めて終了コード3で終わる",
    "ブラウザの操作に失敗したら、そこまでの記録を書いて終了コード3で終わる",
];

#[given(expr = "設定の固まり判定の閾値が 0 である")]
fn freeze_zero(world: &mut QaWorld) {
    world.configure("freeze_threshold: 3", "freeze_threshold: 0");
}

#[then(expr = "判断モデルを呼び出しておらず、対象ページも開いていない")]
async fn nothing_opened(world: &mut QaWorld) {
    world.run().await;
    let ran = world.ran();
    let seen = (world.asked(), ran.browser.observed, world.touched().len());
    assert_eq!(seen, (0, 0, 0), "no question, no page");
}

#[given(expr = "実行の記録の出力先が 空きの無いディスク である")]
const fn disk_full(world: &mut QaWorld) {
    world.scene.store_limit = Some(1);
}

#[given(expr = "実行の記録の出力先が 書き込み権限の無いディレクトリ である")]
const fn directory_locked(world: &mut QaWorld) {
    world.scene.store_limit = Some(1);
}

#[when(expr = "1ステップを実行し、その記録を書こうとする")]
async fn step_and_write(world: &mut QaWorld) {
    world.scene.replies.push_back(Reply::op("wait", 0.9));
    world.run().await;
}

#[then(expr = "実行はそれ以上ステップを進めずに止まっている")]
async fn stopped(world: &mut QaWorld) {
    world.run().await;
    let ran = world.ran();
    let failed = matches!(ran.ending, Some(Ending::Failed(_)));
    let seen = (
        failed,
        world.asked() <= 1,
        world.touched().len(),
        world.report().is_none(),
    );
    assert_eq!(seen, (true, true, 0, true), "stopped at the first write");
}

#[given(expr = "次のブラウザの操作は失敗する")]
fn next_touch_fails(world: &mut QaWorld) {
    world.scene.fail_touch = Some(world.scene.replies.len() + 1);
}

#[when(expr = "判断モデルが {string} のクリックを選び、ブラウザが操作に失敗する")]
async fn click_fails(world: &mut QaWorld, name: String) {
    world.scene.show_button(name.clone());
    world
        .scene
        .replies
        .push_back(Reply::op("click", 0.9).on(name));
    world.run().await;
}

#[then(expr = "実行は終了理由 error で終わっている")]
async fn ended_in_error(world: &mut QaWorld) {
    world.run().await;
    let ending = format!("{:?}", world.ran().ending);
    let last = world.lines().last().map(|line| line["reason"].clone());
    let report = world.report().map(|report| report["reason"].clone());
    let error = Some(json!("error"));
    assert_eq!(
        (ending.as_str(), last, report),
        ("Some(Broke(BrowserError))", error.clone(), error),
        "the browser broke the run"
    );
}
