//! Steps of the Rule 台帳の書式 about the numbers of the issues across
//! runs.

use cucumber::{given, then, when};
use webqa_core::interview::issue::{Kind, Number};

use super::NEXT;
use crate::issues::script::{Step, entries, journey, path_of, screen};
use crate::ledger::{number_of, read_back};
use crate::page::PageSpec;
use crate::scene::found;
use crate::world::{InterviewWorld, Ran};

/// The page of the screen `name` of the scenes of the numbers: a button
/// and a link.
fn order_page(name: &str) -> PageSpec {
    screen(name, &[NEXT]).with(PageSpec::link("戻る"))
}

/// Runs the scene, and keeps its run as the run before, so that the next
/// run is another one of the same scene.
async fn first_run(world: &mut InterviewWorld) {
    world.run().await;
    world.again();
}

/// A run that finds a hesitation about the first element of the page.
async fn hesitated_before(world: &mut InterviewWorld, name: &str, button: &str) {
    assert_eq!(button, NEXT, "the button of the page");
    let steps = [Step::click(1).hesitating(0.9), Step::of("done")];
    journey(&mut world.scene, &[order_page(name)], &steps);
    first_run(world).await;
}

#[given(expr = "前回の実行で、画面 {string} のボタン {string} に判断に迷う課題が出ている")]
async fn hesitated_last_time(world: &mut InterviewWorld, name: String, button: String) {
    hesitated_before(world, &name, &button).await;
}

#[given(expr = "画面 {string} のボタン {string} に、判断に迷う課題がある")]
async fn hesitation_exists(world: &mut InterviewWorld, name: String, button: String) {
    hesitated_before(world, &name, &button).await;
}

#[when(expr = "同じ設定でもう一度実行し、同じ画面の同じ要素に判断に迷う課題が出る")]
async fn runs_again_hesitating(world: &mut InterviewWorld) {
    world.run().await;
}

/// Runs the scene with the page of the screen `name` and the steps.
async fn runs_with(world: &mut InterviewWorld, name: &str, steps: &[Step]) {
    journey(&mut world.scene, &[order_page(name)], steps);
    world.run().await;
}

#[when(expr = "画面 {string} のボタン {string} の判断に迷う の課題が出る")]
async fn other_screen(world: &mut InterviewWorld, name: String, button: String) {
    assert_eq!(button, NEXT, "the button is the same");
    let steps = [Step::click(1).hesitating(0.9), Step::of("done")];
    runs_with(world, &name, &steps).await;
}

#[when(expr = "画面 {string} のリンク {string} の判断に迷う の課題が出る")]
async fn other_element(world: &mut InterviewWorld, name: String, link: String) {
    assert_eq!(link, "戻る", "the link of the page");
    let steps = [Step::click(2).hesitating(0.9), Step::of("done")];
    runs_with(world, &name, &steps).await;
}

#[when(expr = "画面 {string} のボタン {string} の視認性が悪い の課題が出る")]
async fn other_kind(world: &mut InterviewWorld, name: String, button: String) {
    assert_eq!(button, NEXT, "the button is the same");
    world.scene.images = true;
    let steps = [
        Step::click(1).seeing("1: 背景に近い色で目立たない"),
        Step::of("done"),
    ];
    runs_with(world, &name, &steps).await;
}

/// The numbers of the issues of the ledger `run` wrote.
fn numbers_of(ran: &Ran) -> Vec<Number> {
    let text = found(ran.store.ledger.as_deref(), "the run wrote a ledger");
    let ledger = read_back(text);
    ledger
        .entries
        .iter()
        .map(|entry| entry.issue.number)
        .collect()
}

#[then(expr = "2つの課題の番号は違う")]
async fn numbers_differ(world: &mut InterviewWorld) {
    world.run().await;
    let (before, now) = (numbers_of(world.before()), numbers_of(world.ran()));
    assert_eq!((before.len(), now.len()), (1, 1), "one issue each time");
    assert_ne!(before, now, "the numbers differ");
}

#[then(expr = "今回の課題の番号は前回と同じである")]
async fn number_stays(world: &mut InterviewWorld) {
    world.run().await;
    let (before, now) = (numbers_of(world.before()), numbers_of(world.ran()));
    assert_eq!(now.len(), 1, "one issue");
    assert_eq!(now, before, "the number of the run before");
    let entry = found(entries(world).into_iter().next(), "the issue");
    let page = &world.scene.start;
    let elements: Vec<(&str, &str)> = page
        .elements
        .iter()
        .map(|e| (e.role.as_str(), e.name.as_str()))
        .collect();
    let target = entry.issue.target.as_ref();
    let target = target.map(|t| (t.role.as_str(), t.name.as_str(), t.nth));
    let expected = number_of(&page.path, &elements, target, entry.issue.kind());
    assert_eq!(
        now,
        [expected],
        "made of the screen, the element and the kind"
    );
}

#[given(expr = "前回の実行で、画面 {string} に目的に届かない課題が出ている")]
async fn unreached_last_time(world: &mut InterviewWorld, name: String) {
    journey(
        &mut world.scene,
        &[screen(&name, &[NEXT])],
        &[Step::of("stuck")],
    );
    first_run(world).await;
}

#[when(expr = "同じ設定でもう一度実行し、同じ画面で目的に届かない課題が出る")]
async fn runs_again_unreached(world: &mut InterviewWorld) {
    world.run().await;
}

#[then(expr = "画面 {string} の目的のずれの課題が出れば、その番号は目的に届かない課題と違う")]
async fn mismatch_differs(world: &mut InterviewWorld, name: String) {
    world.run().await;
    let mismatch = number_of(path_of(&name), &[("button", NEXT)], None, Kind::Mismatch);
    assert_ne!(
        numbers_of(world.ran()),
        [mismatch],
        "the kind is in the number"
    );
}
