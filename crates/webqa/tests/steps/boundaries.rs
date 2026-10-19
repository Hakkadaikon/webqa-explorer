//! Steps of the rule on the configuration and the outside boundaries.

use cucumber::given;

use crate::judge::Reply;
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &["判断モデルを差し替えても同じ探索ができる"];

/// The page gets a button named `target`, and the stub answers `op` on it.
#[given(
    expr = "判断モデルとして、操作 {string} と対象要素 {string} を返すだけのスタブを設定している"
)]
fn stub(world: &mut QaWorld, op: String, target: String) {
    world.scene.show_button(&target);
    let reply = Reply::op(&op, 1.0).on(&target);
    world.scene.replies.push_back(reply);
    drop((op, target));
}
