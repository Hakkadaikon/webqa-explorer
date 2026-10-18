//! Steps of the background shared by every scenario.

use cucumber::given;

use crate::world::QaWorld;

#[given(expr = "許可ドメイン {string} の対象ページを開いている")]
fn opened(world: &mut QaWorld, origin: String) {
    world.origins.push(origin);
}

#[given(expr = "固まり判定の閾値は {int} 回、行き詰まり判定の閾値は {int} 回に設定されている")]
fn thresholds(world: &mut QaWorld, freeze: u32, stuck: u32) {
    world.limits.insert("freeze_threshold", freeze);
    world.limits.insert("stuck_threshold", stuck);
}

#[given(
    expr = "判断モデルの呼び出し上限は {int} 回、1時間窓あたりの操作回数の上限は {int} 回に設定されている"
)]
fn budgets(world: &mut QaWorld, decisions: u32, requests: u32) {
    world.limits.insert("max_decisions", decisions);
    world.limits.insert("requests_per_hour", requests);
}
