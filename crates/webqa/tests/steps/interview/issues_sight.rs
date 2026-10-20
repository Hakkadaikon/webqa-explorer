//! Steps of the Rule 課題の見つけ方 about the elements hard to see, and
//! about a bridge that says it does not read images.

use cucumber::{given, then};
use serde_json::Value;
use webqa_core::interview::issue::{Detail, Kind, Shot};

use super::script::{
    Step, entries, finished, journey, kinds, ledger, open_journey, reads_images, screen,
};
use crate::ledger::number_of;
use crate::page::PageSpec;
use crate::scene::found;
use crate::world::InterviewWorld;

/// The buttons of the screen of these scenes.
const BUTTONS: [&str; 2] = ["次へ", "クーポンを使う"];

/// The screen of these scenes: the list with the buttons.
fn list() -> Vec<PageSpec> {
    vec![screen("商品一覧", &BUTTONS)]
}

#[given(
    expr = "判断モデルが、スクリーンショットを見て、ボタン {string} を目立たない要素として挙げる"
)]
fn lists_hard_to_see(world: &mut InterviewWorld, name: String) {
    assert_eq!(name, BUTTONS[1], "the button of the second line");
    drop(name);
    let scene = &mut world.scene;
    scene.images = true;
    let step = Step::click(1).seeing("2: 背景に近い色で目立たない");
    journey(scene, &list(), &[step, Step::of("done")]);
}

#[then(expr = "台帳に、{string} を対象とした視認性が悪い課題が出ている")]
async fn hard_to_see_found(world: &mut InterviewWorld, name: String) {
    world.run().await;
    let elements = BUTTONS.map(|button| ("button", button));
    let target = Some(("button", name.as_str(), 0));
    let number = number_of("/products", &elements, target, Kind::Visibility);
    let reason = "背景に近い色で目立たない".to_owned();
    let detail = Detail::Visibility {
        reason,
        expectation: "進む".to_owned(),
    };
    let found: Vec<_> = entries(world)
        .into_iter()
        .map(|e| (e.issue.number, e.issue.detail))
        .collect();
    assert_eq!(found, [(number, detail)], "the issue about the button");
}

#[given(expr = "どのステップでも、判断モデルが見えにくい要素を挙げない")]
fn lists_nothing(world: &mut InterviewWorld) {
    let scene = &mut world.scene;
    scene.images = true;
    journey(scene, &list(), &[Step::click(1), Step::of("done")]);
}

#[then(expr = "台帳に、視認性が悪い課題は出ていない")]
async fn no_hard_to_see(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(finished(world), 3, "the three trials finished");
    let ledger = ledger(world);
    assert_eq!(
        ledger.head.images,
        reads_images(&world.scene),
        "what was said"
    );
    assert_eq!(kinds(world), [], "no issue");
}

#[given(expr = "橋渡しが起動のときに、画像を扱えないと申告する")]
fn bridge_without_images(world: &mut InterviewWorld) {
    let scene = &mut world.scene;
    scene.bridge = Some(false);
    journey(scene, &list(), &[Step::click(1), Step::of("done")]);
}

#[given(expr = "橋渡しが起動のときに、画像を扱えると申告する")]
const fn bridge_with_images(world: &mut InterviewWorld) {
    world.scene.bridge = Some(true);
}

#[then(expr = "台帳に、視認性を確かめなかったことが書かれている")]
async fn sight_unchecked(world: &mut InterviewWorld) {
    world.run().await;
    assert!(!ledger(world).head.images, "it was not checked");
}

#[given(expr = "判断に迷う課題が起きるステップで、画面のスクリーンショットを撮っている")]
fn hesitates_at_shot(world: &mut InterviewWorld) {
    let step = Step::click(1).hesitating(0.9);
    journey(&mut world.scene, &list(), &[step, Step::of("done")]);
}

#[then(expr = "橋渡しに書いた問いの JSON に、画像は含まれていない")]
async fn asked_without_image(world: &mut InterviewWorld) {
    world.run().await;
    let asked = &world.ran().bridge;
    assert!(!asked.is_empty(), "the bridge was asked");
    let with: Vec<&Value> = asked
        .iter()
        .filter(|line| line.get("image").is_some())
        .collect();
    assert_eq!(with, Vec::<&Value>::new(), "no line carries an image");
}

#[then(expr = "台帳のその課題には、撮ったスクリーンショットの場所が書かれている")]
async fn shot_located(world: &mut InterviewWorld) {
    world.run().await;
    let issues: Vec<_> = entries(world).into_iter().map(|e| e.issue).collect();
    let at = Shot { trial: 1, step: 1 };
    let shots: Vec<Option<Shot>> = issues.iter().map(|issue| issue.shot).collect();
    assert_eq!(shots, [Some(at)], "where the ledger says the shot is");
    let number = found(issues.first(), "the issue").number;
    let names: Vec<String> = world
        .ran()
        .store
        .shots
        .iter()
        .map(|(name, _)| name.clone())
        .collect();
    assert_eq!(
        names,
        [format!("{number}_trial1_step1.png")],
        "the shot that was taken"
    );
}

#[given(expr = "判断モデルの応答に、見えにくい要素の答えが無い")]
fn answers_no_sight(world: &mut InterviewWorld) {
    let step = Step::click(1).without_sight();
    open_journey(&mut world.scene, &list(), &[step]);
}

#[then(expr = "応答の異常として記録されていない")]
async fn no_anomaly(world: &mut InterviewWorld) {
    world.run().await;
    assert!(
        !world.kinds().contains(&"anomaly".to_owned()),
        "the log has none"
    );
}

#[then(expr = "判断モデルが選んだ操作を、ガードレールを通したあとに実行している")]
async fn operated(world: &mut InterviewWorld) {
    world.run().await;
    let ops: Vec<String> = world
        .ran()
        .touches
        .iter()
        .map(|(_, t)| t.op.clone())
        .collect();
    assert_eq!(ops, ["click"], "the chosen operation ran");
}
