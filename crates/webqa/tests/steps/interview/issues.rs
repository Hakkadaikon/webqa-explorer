//! Steps of the Rule 課題の見つけ方: the clues of hesitation, the elements
//! hard to see, and the purposes a trial did not reach.

use cucumber::{given, then, when};
use webqa_core::interview::ask::HESITATION;
use webqa_core::interview::clue::Clue;
use webqa_core::interview::issue::Detail;
use webqa_core::judge::{Answers, QuestionId};

use crate::page::PageSpec;
use crate::scene::{Scene, found};
use crate::world::InterviewWorld;

#[path = "issues_reach.rs"]
mod reach;
#[path = "issues_script.rs"]
pub mod script;
#[path = "issues_sight.rs"]
mod sight;
#[path = "issues_trail.rs"]
mod trail;
use script::{Step, entries, finished, journey, kinds, screen};

/// The scenarios of this Rule that have their steps and pass. Two more have
/// their steps, and wait for the product: with a bridge that reads no
/// images no screenshot is taken though the ledger says where it is
/// ("橋渡しが画像を扱えないと申告したら、問い合わせにスクリーンショットを含めない"),
/// and an answer without the elements hard to see is no anomaly
/// ("橋渡しが画像を扱えると申告したのに見えにくい要素の答えが無ければ、応答の異常として扱う").
pub const READY: &[&str] = &[
    "迷いの度合いが閾値以上なら、判断に迷う課題の手がかりにする",
    "確信度が閾値に届かない操作を選んだら、判断に迷う課題の手がかりにする",
    "引き返したら、判断に迷う課題の手がかりにする",
    "同じ画面を行き来したら、行き詰まりとは別に迷いの手がかりとして記録する",
    "同じ画面を行き来しても、試行は行き詰まりで終わらずに続く",
    "目的に届かなかった試行は、止まった画面と理由を目的に届かない課題として記録する",
    "判断モデルが見えにくい要素を挙げたら、視認性が悪い課題として記録する",
    "判断モデルが見えにくい要素を挙げなければ、視認性が悪い課題は出さない",
    "橋渡しが画像を扱えないと申告したら、視認性が悪い課題を出さず、そのことを台帳に書く",
    "橋渡しが画像を扱えないと申告したら、見えにくい要素の答えが無くても応答の異常にしない",
];

/// The option the judge gave to `id` first, with its probability.
fn first_option(answers: &Answers, id: &str) -> (String, f64) {
    let options = answers.table.get(&QuestionId::new(id));
    let first = options.and_then(|options| options.first());
    found(first.cloned(), "the answers have the question")
}

/// The operations the steps of the scene make on the browser: those the
/// judge answers at each step of each trial, but done and stuck.
fn expected_ops(scene: &Scene) -> Vec<String> {
    let steps = scene
        .replies
        .iter()
        .filter_map(|reply| reply.as_ref().ok())
        .filter(|answers| answers.table.contains_key(&QuestionId::new("operation")));
    let ops = steps.map(|answers| first_option(answers, "operation").0);
    ops.filter(|op| !["done", "stuck"].contains(&op.as_str()))
        .collect()
}

/// The operations the browsers of the trials were told to make.
fn executed_ops(world: &InterviewWorld) -> Vec<String> {
    let touches = &world.ran().touches;
    touches.iter().map(|(_, touch)| touch.op.clone()).collect()
}

/// The clues of hesitation the first step of the scene should leave, as
/// the examples of the feature define them, and a step that goes back.
fn expected_clues(scene: &Scene) -> Vec<Clue> {
    let replies: Vec<&Answers> = scene
        .replies
        .iter()
        .filter_map(|reply| reply.as_ref().ok())
        .collect();
    let first = found(replies.first(), "the scene has a step");
    let (_, confidence) = first_option(first, "operation");
    let (_, degree) = first_option(first, HESITATION);
    let steps = replies
        .iter()
        .filter(|answers| answers.table.contains_key(&QuestionId::new("operation")));
    let back = steps
        .clone()
        .any(|answers| first_option(answers, "operation").0 == "back");
    [
        (Clue::Hesitant, degree >= scene.hesitation),
        (Clue::LowConfidence, confidence < scene.confidence),
        (Clue::WentBack, back),
    ]
    .into_iter()
    .filter_map(|(clue, hit)| hit.then_some(clue))
    .collect()
}

#[given(expr = "判断モデルが迷いの度合いを {float} で答え、確信度 {float} で {string} を選ぶ")]
fn chooses(world: &mut InterviewWorld, degree: f64, confidence: f64, name: String) {
    let page = [screen("商品一覧", &[&name])];
    let step = Step::click(1).hesitating(degree).sure(confidence);
    journey(&mut world.scene, &page, &[step, Step::of("done")]);
    drop(name);
}

#[then(expr = "そのステップは判断に迷う手がかりとして記録されている")]
async fn clue_recorded(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(finished(world), 3, "the three trials finished");
    assert_eq!(
        executed_ops(world),
        expected_ops(&world.scene),
        "the operations"
    );
    let clues = expected_clues(&world.scene);
    let expectation = "進む".to_owned();
    let details: Vec<Detail> = entries(world)
        .into_iter()
        .map(|entry| entry.issue.detail)
        .collect();
    assert_eq!(
        details,
        [Detail::Hesitant { clues, expectation }],
        "the issue"
    );
}

#[then(expr = "そのステップは判断に迷う手がかりとして記録されていない")]
async fn clue_not_recorded(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(finished(world), 3, "the three trials finished");
    assert_eq!(
        executed_ops(world),
        expected_ops(&world.scene),
        "the operations"
    );
    assert_eq!(kinds(world), [], "no issue");
}

/// The screens of the scenario that goes back: the list, the detail, and
/// the list again after going back.
fn back_pages() -> Vec<PageSpec> {
    let list = screen("商品一覧", &["詳細を見る"]);
    vec![list.clone(), screen("商品詳細", &["カートに入れる"]), list]
}

#[given(expr = "前のステップで画面 {string} から画面 {string} へ移った")]
fn moved(world: &mut InterviewWorld, from: String, to: String) {
    let screens = (from.as_str(), to.as_str());
    assert_eq!(screens, ("商品一覧", "商品詳細"), "the two screens");
    drop((from, to));
    journey(
        &mut world.scene,
        &back_pages(),
        &[Step::click(1), Step::of("done")],
    );
}

#[when(expr = "判断モデルが back を返し、ガードレールを通って実行される")]
fn goes_back(world: &mut InterviewWorld) {
    let steps = [Step::click(1), Step::of("back"), Step::of("done")];
    journey(&mut world.scene, &back_pages(), &steps);
}
