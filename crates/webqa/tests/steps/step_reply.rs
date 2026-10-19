//! Steps of the rule on one step, part: replies that cannot be used.

use cucumber::{given, then};
use webqa_core::judge::JudgeError;

use crate::judge::Reply;
use crate::world::QaWorld;

/// The page shows a button, and the judge gives `reply`.
fn with_button(world: &mut QaWorld, reply: Reply) {
    world.scene.show_button("保存");
    world.scene.replies.push_back(reply);
}

#[given(expr = "判断モデルの応答が 存在しない要素番号 99 を対象にした click である")]
fn missing_number(world: &mut QaWorld) {
    with_button(world, Reply::op("click", 0.9).on_number(99));
}

#[given(expr = "判断モデルの応答が 操作可能な要素が1つも無い画面で返った click である")]
fn click_without_elements(world: &mut QaWorld) {
    world
        .scene
        .replies
        .push_back(Reply::op("click", 0.9).on_number(1));
}

#[given(expr = "判断モデルの応答が 8種のどれでもない操作種別 {string} である")]
fn unknown_operation(world: &mut QaWorld, op: String) {
    with_button(world, Reply::op(op, 0.9));
}

#[given(expr = "判断モデルの応答が 必要な項目が欠けた形式の崩れた応答 である")]
fn malformed(world: &mut QaWorld) {
    with_button(world, Reply::malformed());
}

#[given(expr = "判断モデルの応答が API のエラー応答 である")]
fn api_error(world: &mut QaWorld) {
    with_button(world, Reply::Fail(JudgeError::Api));
}

#[given(expr = "判断モデルの応答が 設定の待ち時間内に返らないタイムアウト である")]
fn timed_out(world: &mut QaWorld) {
    with_button(world, Reply::Fail(JudgeError::Timeout));
}

#[then(expr = "ブラウザ操作をしていない")]
async fn did_nothing(world: &mut QaWorld) {
    world.run().await;
    assert_eq!(world.touched(), [], "the browser was not operated");
}

#[then(expr = "応答の異常として記録されている")]
async fn anomaly_logged(world: &mut QaWorld) {
    world.run().await;
    assert!(world.kinds().contains(&"anomaly".to_owned()), "an anomaly");
}

#[then(expr = "判断モデルの呼び出し回数は {int} 増えている")]
async fn answered_more(world: &mut QaWorld, times: usize) {
    world.run().await;
    assert_eq!(world.answered(), times, "the replies of the judge");
}
