//! Steps of the Rule 課題の見つけ方 about going back and forth between two
//! screens.

use cucumber::{given, then, when};
use serde_json::Value;
use webqa_core::interview::clue::Clue;
use webqa_core::interview::issue::{Detail, Kind, Number};

use super::script::{Step, endings, entries, journey, open_journey, screen};
use crate::ledger::number_of;
use crate::page::PageSpec;
use crate::world::InterviewWorld;

/// The pages of two screens whose visits alternate, with buttons for as
/// many visits as the scenarios make, so that every operation is on an
/// element not operated before.
fn two_screens() -> Vec<PageSpec> {
    let list = screen(
        "商品一覧",
        &["商品を見る", "ほかの商品を見る", "人気の商品を見る"],
    );
    let detail = screen("商品詳細", &["カートに入れる", "レビューを見る"]);
    vec![list.clone(), detail.clone(), list, detail]
}

/// The clicks of the four visits, each on an element not clicked before.
fn four_clicks() -> Vec<Step> {
    [1, 1, 2, 2].map(Step::click).to_vec()
}

#[given(expr = "試行の中で画面 {string} と画面 {string} の間を移っている")]
fn between(world: &mut InterviewWorld, first: String, second: String) {
    let screens = (first.as_str(), second.as_str());
    assert_eq!(screens, ("商品一覧", "商品詳細"), "the two screens");
    drop((first, second));
    let steps = [four_clicks(), vec![Step::of("done")]].concat();
    journey(&mut world.scene, &two_screens(), &steps);
}

#[when(expr = "操作の結果、{string}、{string}、{string}、{string} の順に画面が移る")]
fn moves_in_order(world: &mut InterviewWorld, a: String, b: String, c: String, d: String) {
    let order = ["商品一覧", "商品詳細", "商品一覧", "商品詳細"];
    assert_eq!([a, b, c, d], order, "the screens of the four steps");
    world.scene.stuck_threshold = 1;
}

#[then(expr = "2つの画面の行き来が、判断に迷う手がかりとして記録されている")]
async fn ping_pong_recorded(world: &mut InterviewWorld) {
    world.run().await;
    let elements = [("button", "カートに入れる"), ("button", "レビューを見る")];
    let number = |target| number_of("/products/1", &elements, target, Kind::Hesitant);
    let detail = Detail::Hesitant {
        clues: vec![Clue::PingPong],
        expectation: "進む".to_owned(),
    };
    let review = Some(("button", "レビューを見る", 0));
    let mut expected = vec![(number(review), detail.clone()), (number(None), detail)];
    expected.sort_by_key(|(number, _)| *number);
    let mut found: Vec<(Number, Detail)> = entries(world)
        .into_iter()
        .map(|entry| (entry.issue.number, entry.issue.detail))
        .collect();
    found.sort_by_key(|(number, _)| *number);
    assert_eq!(found, expected, "the issues of the two screens");
}

#[then(expr = "行き詰まりの回数は、行き来を記録したことでは増えていない")]
async fn stuck_not_counted(world: &mut InterviewWorld) {
    world.run().await;
    let done = Value::from("done");
    assert_eq!(
        endings(world),
        [done.clone(), done.clone(), done],
        "all ended by done"
    );
}

#[given(expr = "行き詰まりの閾値は {int} 回に設定されている")]
const fn stuck_at(world: &mut InterviewWorld, threshold: u32) {
    world.scene.stuck_threshold = threshold;
}

#[given(expr = "操作のたびに未操作の要素を操作していて、行き詰まりの回数は {int} である")]
fn operates_new(world: &mut InterviewWorld, count: u32) {
    assert_eq!(count, 0, "no step without progress yet");
    let pages = [
        two_screens(),
        vec![screen(
            "商品一覧",
            &["商品を見る", "ほかの商品を見る", "人気の商品を見る"],
        )],
    ]
    .concat();
    open_journey(&mut world.scene, &pages, &four_clicks());
}

#[when(expr = "同じ2つの画面の間を {int} 回行き来する")]
const fn goes_between(_world: &mut InterviewWorld, times: usize) {
    assert!(times == 4, "the clicks go four times");
}

#[then(expr = "試行は終わっておらず、次のステップで判断モデルに問い合わせる")]
async fn trial_goes_on(world: &mut InterviewWorld) {
    world.run().await;
    let ran = world.ran();
    assert_eq!(ran.touches.len(), 4, "the four operations ran");
    assert_eq!(ran.judge.asked.len(), 5, "the fifth step asked the judge");
    assert!(ran.judge.stalled, "and found no answer");
    assert_eq!(endings(world), [Value::from("interrupted")], "only the cut");
}
