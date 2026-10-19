//! Steps of the rule on the configuration and the outside boundaries.

use std::mem;

use cucumber::{given, then, when};
use webqa_core::judge::QuestionId;
use webqa_core::machine::ExitReason;

use crate::world::QaWorld;

/// The page gets a button named `target`, and the stub answers `op` on it.
#[given(
    expr = "判断モデルとして、操作 {string} と対象要素 {string} を返すだけのスタブを設定している"
)]
fn stub(world: &mut QaWorld, op: String, target: String) {
    world.elements.push(("button".to_owned(), target));
    let number = world.elements.len().to_string();
    let script = mem::take(&mut world.judge.script);
    world.judge.script = script
        .then(QuestionId::operation(), [(op, 1.0)])
        .then(QuestionId::target(), [(number, 1.0)])
        .then(QuestionId::error_shown(), [("true", 0.0)]);
}

/// The script answers once, so every later question is an anomaly until the
/// decisions run out.
#[when(expr = "1ステップを実行する")]
async fn one_step(world: &mut QaWorld) {
    let reason = world.run().await.map(|outcome| outcome.reason);
    assert_eq!(
        reason,
        Ok(ExitReason::BudgetExceeded),
        "after the step, the script runs out and so do the decisions"
    );
}

#[then(expr = "ブラウザで {string} をクリックしている")]
fn clicked(world: &mut QaWorld, name: String) {
    let touched: Vec<(String, Option<String>)> = mem::take(&mut world.browser.touches)
        .into_iter()
        .map(|t| (t.op, t.target.map(|id| id.name)))
        .collect();
    assert_eq!(
        touched,
        vec![("click".to_owned(), Some(name))],
        "clicked once"
    );
}
