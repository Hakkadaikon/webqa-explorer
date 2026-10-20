//! Steps of the Rule 課題の見つけ方 about a trial that ends without
//! reaching the purpose.

use cucumber::{given, then};
use webqa_core::interview::issue::{Detail, Halt, Kind, Number};

use super::script::{Step, entries, journey, path_of, screen};
use crate::ledger::number_of;
use crate::world::InterviewWorld;

#[given(expr = "試行が画面 {string} で行き詰まりとして終わる")]
fn gives_up(world: &mut InterviewWorld, name: String) {
    let page = [screen(&name, &["次へ"])];
    journey(&mut world.scene, &page, &[Step::of("stuck")]);
    drop(name);
}

/// The number of the issue of `kind` about no element on the screen `name`
/// of the scenes of this Rule: one button, "次へ".
fn unreached_number(name: &str) -> Number {
    number_of(path_of(name), &[("button", "次へ")], None, Kind::Unreached)
}

#[then(expr = "台帳に、画面 {string} の目的に届かない課題が出ている")]
async fn unreached_found(world: &mut InterviewWorld, name: String) {
    world.run().await;
    let found: Vec<Number> = entries(world).iter().map(|e| e.issue.number).collect();
    assert_eq!(found, [unreached_number(&name)], "the issue of the screen");
}

#[then(expr = "その課題に、最後のステップの期待は書かれていない")]
async fn no_expectation_written(world: &mut InterviewWorld) {
    world.run().await;
    let halt = Halt::GaveUp;
    // The step that gave up carries out no operation: no expectation.
    let expected = Detail::Unreached {
        halt,
        expectation: Some(String::new()),
    };
    let details: Vec<Detail> = entries(world).into_iter().map(|e| e.issue.detail).collect();
    assert_eq!(details, [expected], "the last step asked no expectation");
}
