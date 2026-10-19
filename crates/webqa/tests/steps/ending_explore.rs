//! Steps of the Rule 終了の判定: exploring. The scenarios whose steps are
//! all here are named in `READY`.

use cucumber::{given, then};
use webqa_core::judge::{Question, QuestionId};
use webqa_core::machine::ExitReason;

use crate::ending::{explore, held, written_reasons};
use crate::judge::Reply;
use crate::page::PageSpec;
use crate::remembered::remember;
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "探索モードで禁止要素を除くすべての要素を探索し終えたら、ゴール達成で終了する",
    "探索モードで未操作の要素が禁止要素だけになったら、ゴール達成で終了する",
    "探索モードで禁止要素以外の未操作の要素が残っていれば、ゴール達成にしない",
    "探索モードで探索してよい要素が無くても、最初の問い合わせの前にはゴール達成にしない",
    "探索モードで探索してよい要素が無ければ、1回問い合わせた後にゴール達成で終了する",
    "探索モードでは、判断モデルに問い合わせる操作種別の選択肢に done を入れない",
    "探索モードで判断モデルが done を返しても、未検証の終了にせず応答の異常として扱う",
];

#[given(expr = "探索モードで実行している")]
fn explores(world: &mut QaWorld) {
    explore(&mut world.scene);
}

#[given(expr = "開始の画面から辿れるすべての画面で、すべての要素を探索し終えている")]
fn everything_explored(world: &mut QaWorld) {
    world.scene.start.elements = vec![PageSpec::button("保存")];
    let screens: &[(&str, &[(&str, &str)])] = &[("/", &[("button", "保存")])];
    remember(&mut world.scene, screens, &[(0, "保存", 0)]);
}

#[given(expr = "判断モデルに1回以上問い合わせている")]
fn asked_once(world: &mut QaWorld) {
    held(world, 1);
}

#[given(expr = "開始の画面から辿れる画面に残る未操作の要素は、禁止要素 {string} だけである")]
fn only_forbidden_left(world: &mut QaWorld, name: String) {
    let elements = vec![PageSpec::button("保存"), PageSpec::button(&name)];
    world.scene.start.elements = elements;
    let screens: &[(&str, &[(&str, &str)])] = &[("/", &[("button", "保存"), ("button", "退会")])];
    remember(&mut world.scene, screens, &[(0, "保存", 0)]);
    drop(name);
}

#[then(expr = "{string} を操作していない")]
async fn did_not_operate(world: &mut QaWorld, name: String) {
    world.run().await;
    let mut targets = world.touched().into_iter().map(|(_, target)| target);
    assert!(
        !targets.any(|target| target == Some(name.clone())),
        "left alone"
    );
}

#[given(expr = "今の画面 {string} の要素はすべて探索し終えている")]
fn this_screen_explored(world: &mut QaWorld, title: String) {
    world.scene.start.elements = vec![PageSpec::button("保存")];
    world.scene.marks.insert("top", 1);
    drop(title);
}

#[given(expr = "{string} から辿れる画面 {string} に未操作の要素 {string} が残っている")]
fn another_screen_left(world: &mut QaWorld, from: String, to: String, name: String) {
    let screens: &[(&str, &[(&str, &str)])] = &[
        ("/", &[("button", "保存")]),
        ("/settings", &[("button", &name)]),
    ];
    remember(&mut world.scene, screens, &[(0, "保存", 1)]);
    drop((from, to, name));
}

#[then(expr = "実行は終了しておらず、判断モデルに問い合わせている")]
async fn asks_and_goes_on(world: &mut QaWorld) {
    world.run().await;
    assert_eq!(world.reason(), None, "the run has not ended");
    assert!(world.ran().judge.stalled, "the judge was asked");
}

#[given(expr = "開始の画面には操作可能な要素が1つも無く、ほかの画面へは辿れない")]
fn nothing_to_operate(world: &mut QaWorld) {
    world.scene.start.elements.clear();
}

#[given(expr = "開始の画面には操作可能な要素が禁止要素 {string} だけあり、ほかの画面へは辿れない")]
fn only_forbidden_to_operate(world: &mut QaWorld, name: String) {
    world.scene.start.elements = vec![PageSpec::button(&name)];
    drop(name);
}

#[given(expr = "判断モデルに1回問い合わせ、その応答はscroll_downで、探索済みの要素は増えていない")]
fn answered_by_scrolling(world: &mut QaWorld) {
    world.scene.replies.push_back(Reply::op("scroll_down", 0.9));
}

#[given(
    expr = "判断モデルに1回問い合わせ、その応答は禁止要素 {string} への clickで、探索済みの要素は増えていない"
)]
fn answered_by_forbidden_click(world: &mut QaWorld, name: String) {
    world
        .scene
        .replies
        .push_back(Reply::op("click", 0.9).on(name));
}

#[given(
    expr = "判断モデルに1回問い合わせ、その応答は形式の崩れた応答による応答の異常で、探索済みの要素は増えていない"
)]
fn answered_malformed(world: &mut QaWorld) {
    world.scene.replies.push_back(Reply::malformed());
}

/// The options of the question about the operation, in the first call.
fn operation_options(world: &QaWorld) -> Vec<String> {
    let first = world
        .ran()
        .judge
        .questions
        .first()
        .cloned()
        .unwrap_or_default();
    let options = first.into_iter().map(|question| match question {
        Question::Choice { id, options, .. } if id == QuestionId::operation() => options,
        Question::Choice { .. }
        | Question::Noul { .. }
        | Question::Score { .. }
        | Question::Text { .. } => Vec::new(),
    });
    options.flatten().collect()
}

#[then(
    expr = "判断モデルに問い合わせた操作種別の選択肢は click, input, scroll_down, scroll_up, back, wait, stuck の7種である"
)]
async fn seven_options(world: &mut QaWorld) {
    world.run().await;
    let seven = [
        "click",
        "input",
        "scroll_down",
        "scroll_up",
        "back",
        "wait",
        "stuck",
    ];
    assert_eq!(operation_options(world), seven, "the options");
}

#[then(expr = "その選択肢に done は含まれない")]
async fn no_done_option(world: &mut QaWorld) {
    world.run().await;
    assert!(
        !operation_options(world).contains(&"done".to_owned()),
        "no done"
    );
}

#[given(expr = "判断モデルが選択肢に無い操作 {string} を返す")]
fn replies_outside_options(world: &mut QaWorld, op: String) {
    world.scene.replies.push_back(Reply::op(op, 0.9));
}

#[then(expr = "実行は未検証の終了として記録されていない")]
async fn not_recorded_unverified(world: &mut QaWorld) {
    world.run().await;
    assert_ne!(world.reason(), Some(ExitReason::ReportedDone), "the reason");
    let said = written_reasons(world).contains(&"reported_done".to_owned());
    assert!(!said, "no line says it was reported done");
}

#[then(expr = "応答の異常として記録され、実行は終了していない")]
async fn recorded_as_anomaly(world: &mut QaWorld) {
    world.run().await;
    let anomalies: Vec<_> = world
        .lines()
        .into_iter()
        .filter(|line| line.get("kind") == Some(&"anomaly".into()))
        .map(|line| (line.get("cause").cloned(), line.get("question").cloned()))
        .collect();
    let expected = (Some("not_an_option".into()), Some("operation".into()));
    assert_eq!(
        anomalies,
        [expected],
        "one anomaly: not an option of the operation"
    );
    assert_eq!(world.reason(), None, "the run has not ended");
}
