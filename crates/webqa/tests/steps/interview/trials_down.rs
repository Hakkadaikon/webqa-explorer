//! Steps of a trial that the judge's failures in a row cut.

use cucumber::{given, then};
use webqa_core::judge::JudgeError;

use super::{End, course, ending_of, fill, first};
use crate::world::InterviewWorld;

#[given(
    expr = "1回目の試行が行き詰まりで終わり、2回目の試行で判断モデルの呼び出しが {int} 回続けて失敗する"
)]
fn second_judge_down(world: &mut InterviewWorld, failures: usize) {
    let scene = &mut world.scene;
    first(scene, 0, End::GaveUp);
    let call = course(scene, 0, End::Cut);
    for reply in scene.replies.iter_mut().skip(call).take(failures) {
        *reply = Err(JudgeError::Api { detail: None });
    }
    fill(scene);
}

#[then(expr = "2回目の試行は判断モデルの停止による打ち切りで終わっている")]
async fn second_cut_by_judge(world: &mut InterviewWorld) {
    world.run().await;
    let ending = ending_of(world, 2);
    assert_eq!(ending.as_deref(), Some("judge_down"), "how it ended");
}
