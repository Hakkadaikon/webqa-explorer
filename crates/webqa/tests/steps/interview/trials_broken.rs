//! Steps of the trials on a browser that fails in the middle of a trial.

use cucumber::{given, then};
use serde_json::json;
use webqa::interview::trials::Ending;
use webqa_core::ports::BrowserError;

use super::{End, course, first};
use crate::world::InterviewWorld;

#[given(expr = "試行は 2 回で、2回目の試行の途中でブラウザの操作が失敗する")]
fn second_breaks(world: &mut InterviewWorld) {
    let scene = &mut world.scene;
    scene.trials = 2;
    first(scene, 1, End::Done);
    course(scene, 2, End::Done);
    scene.broken_trial = Some(2);
}

#[then(expr = "実行結果の終了理由は error で、イベントログは終了のイベントで終わっている")]
async fn ended_in_error(world: &mut InterviewWorld) {
    world.run().await;
    let report = world.ran().store.report.as_ref();
    let reason = report.map(|report| report["reason"].clone());
    let last = world.kinds().last().cloned();
    assert_eq!(
        (world.ran().ending, reason, last.as_deref()),
        (
            Some(Ending::Broke(BrowserError)),
            Some(json!("error")),
            Some("run_finished")
        ),
        "the browser broke the run"
    );
}

#[then(expr = "2回目の試行で行った操作も、実行結果の操作の記録に入っている")]
async fn second_in_report(world: &mut InterviewWorld) {
    world.run().await;
    let ran = world.ran();
    let report = ran.store.report.as_ref();
    let steps = report.and_then(|report| report["steps"].as_array().map(Vec::len));
    let second = ran.touches.iter().filter(|(trial, _)| *trial == 2).count();
    assert_eq!(
        (steps, second),
        (Some(ran.touches.len()), 1),
        "every operation, one of them in the second trial"
    );
}
