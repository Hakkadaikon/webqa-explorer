//! The re-run after a run whose trials stopped on a screen because the
//! judge ran out of budget there.

use cucumber::{given, when};
use webqa_core::interview::issue::{Halt, Kind};

use super::course::ticked;
use super::given::hand_over;
use crate::issues::script::{Step, journey, path_of, screen};
use crate::ledger::number_of;
use crate::scene::found;
use crate::world::InterviewWorld;

/// The one button on the screens of these runs.
const BUTTON: &str = "次へ";

#[given(
    expr = "前回の台帳に、判断予算切れで止まった画面 {string} の目的に届かない課題 {string} があり、選んでいる"
)]
async fn before_spent(world: &mut InterviewWorld, name: String, issue: String) {
    journey(
        &mut world.scene,
        &[screen(&name, &[BUTTON])],
        &[Step::of("stuck")],
    );
    let number = number_of(path_of(&name), &[("button", BUTTON)], None, Kind::Unreached);
    world.scene.aliases.bind(&issue, number);
    hand_over(world).await;
    let text = found(world.scene.previous.as_deref(), "handed over");
    let spent = text.replace(Halt::GaveUp.label(), Halt::BudgetSpent.label());
    assert_ne!(spent, text, "the halt was written in the ledger");
    world.scene.previous = Some(ticked(&spent, number));
    drop((name, issue));
}

#[when(expr = "前回の台帳を渡して再実行し、どの試行も {string} に届いて、{string} は1回も起きない")]
fn again_reaching_all(world: &mut InterviewWorld, name: String, issue: String) {
    let steps = [Step::click(1), Step::of("done")];
    journey(&mut world.scene, &[screen(&name, &[BUTTON])], &steps);
    drop((name, issue));
}
