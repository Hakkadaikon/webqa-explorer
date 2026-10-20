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

/// The buttons of the list and of the detail: one for each visit the
/// scenarios make, so that every operation is on an element not operated
/// before.
const LIST: [&str; 4] = [
    "商品を見る",
    "ほかの商品を見る",
    "人気の商品を見る",
    "新着の商品を見る",
];
const DETAIL: [&str; 4] = [
    "カートに入れる",
    "レビューを見る",
    "仕様を見る",
    "在庫を見る",
];

/// The first `visits` pages of two screens whose visits alternate, the
/// list first.
fn alternating(visits: usize) -> Vec<PageSpec> {
    let (list, detail) = (screen("商品一覧", &LIST), screen("商品詳細", &DETAIL));
    vec![list, detail]
        .into_iter()
        .cycle()
        .take(visits)
        .collect()
}

/// The clicks of `visits` visits, each on an element not clicked before:
/// the visits of a screen take its buttons in order.
fn clicks(visits: usize) -> Vec<Step> {
    (0..visits)
        .map(|visit| Step::click(visit / 2 + 1))
        .collect()
}

/// The moves of the scenarios: four round trips between the two screens.
const MOVES: usize = 8;

#[given(expr = "試行の中で画面 {string} と画面 {string} の間を移っている")]
fn between(world: &mut InterviewWorld, first: String, second: String) {
    let screens = (first.as_str(), second.as_str());
    assert_eq!(screens, ("商品一覧", "商品詳細"), "the two screens");
    drop((first, second));
    let steps = [clicks(MOVES), vec![Step::of("done")]].concat();
    journey(&mut world.scene, &alternating(MOVES + 1), &steps);
    // Not in the feature: the moves are more than the 3 steps without progress that end a trial, so a trial that goes on shows the back and forth was not counted.
    world.scene.stuck_threshold = 3;
}

#[when(expr = "操作の結果、{string}、{string}、{string}、{string} の順に画面が移る")]
fn moves_in_order(_world: &mut InterviewWorld, a: String, b: String, c: String, d: String) {
    let order = ["商品一覧", "商品詳細", "商品一覧", "商品詳細"];
    assert_eq!([a, b, c, d], order, "the screens of the first four steps");
}

/// The number of the hesitation about the button `n` of the screen at
/// `path` with the buttons `names`, or about no element.
fn hesitation(path: &str, names: &[&str; 4], n: Option<usize>) -> Number {
    let elements = names.map(|name| ("button", name));
    let target = n
        .and_then(|n| names.get(n))
        .map(|name| ("button", *name, 0));
    number_of(path, &elements, target, Kind::Hesitant)
}

/// The numbers of the issues of the back and forth, which shows from the
/// fourth step on: on the detail and the list by turns, on the element
/// each step clicks, and the last step on the list with none.
fn back_and_forth() -> Vec<Number> {
    let on_detail = |n| hesitation("/products/1", &DETAIL, n);
    let on_list = |n| hesitation("/products", &LIST, n);
    [
        on_detail(Some(1)),
        on_list(Some(2)),
        on_detail(Some(2)),
        on_list(Some(3)),
        on_detail(Some(3)),
        on_list(None),
    ]
    .to_vec()
}

#[then(expr = "2つの画面の行き来が、判断に迷う手がかりとして記録されている")]
async fn ping_pong_recorded(world: &mut InterviewWorld) {
    world.run().await;
    let hesitation = Detail::Hesitant {
        clues: vec![Clue::PingPong],
        expectation: "進む".to_owned(),
    };
    let mut expected: Vec<(Number, Detail)> = back_and_forth()
        .into_iter()
        .map(|number| (number, hesitation.clone()))
        .collect();
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
    open_journey(&mut world.scene, &alternating(5), &clicks(4));
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
