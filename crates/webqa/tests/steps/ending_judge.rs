//! Steps of the Rule 終了の判定: the judge failing several times in a row.
//! The scenarios whose steps are all here are named in `READY`.

use cucumber::{given, when};
use webqa_core::judge::JudgeError;

use crate::ending::explore;
use crate::judge::Reply;
use crate::scene::found;
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] =
    &["判断モデルの失敗が3回続いたら、予算を使い切る前に終了コード3で終わる"];

/// The failures as the feature names them.
const FAILURES: [(&str, JudgeError); 2] = [
    ("API のエラー", JudgeError::Api),
    ("タイムアウト", JudgeError::Timeout),
];

#[given(
    regex = r"^(決め打ちシナリオ|探索モード)で、判断モデルの呼び出しが毎回 (API のエラー|タイムアウト) で失敗する$"
)]
fn judge_keeps_failing(world: &mut QaWorld, mode: String, failure: String) {
    let scene = &mut world.scene;
    scene.show_button("保存");
    if mode == "探索モード" {
        explore(scene);
    }
    let named = FAILURES.iter().find(|(name, _)| *name == failure);
    let (_, error) = found(named, "a failure of the feature");
    for _ in 0..20 {
        scene.replies.push_back(Reply::Fail(error.clone()));
    }
    drop((mode, failure));
}

#[when(expr = "実行を終わりまで進める")]
async fn run_to_the_end(world: &mut QaWorld) {
    world.run().await;
}
