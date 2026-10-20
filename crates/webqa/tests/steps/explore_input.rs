//! Steps of the Rule on the memory of the exploration: what `input` does in
//! explore mode when no value is set for an element.

use cucumber::{given, then};
use serde_json::Value;

use crate::ending::explore;
use crate::ending_explore::operation_options;
use crate::page::{ElementSpec, PageSpec};
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "探索モードでは、値の設定された入力欄が候補にあるときだけ、操作種別の選択肢に input を含める",
    "ページに禁止要素があれば、操作種別の選択肢に enter を含めない",
    "何も打ち込まなかった input は、skipped として記録し、探索済みの要素に数えない",
    "値の設定された入力欄に打てた input は、skipped を記録せず、探索済みの要素に数える",
];

/// The element named in quotes after `kind`, and whether it has a value.
fn element(kind: &str, name: &str) -> (ElementSpec, Option<String>) {
    if kind.contains("値の設定されていない入力欄") {
        (PageSpec::textbox(name), None)
    } else if kind.contains("値の設定された入力欄") {
        (PageSpec::textbox(name), Some(name.to_owned()))
    } else {
        (PageSpec::button(name), None)
    }
}

/// The elements of a phrase such as `ボタン "保存" と入力欄 "備考"`: each
/// quoted name, with the words before it saying what it is.
#[given(regex = r"^開始の画面の要素は(.+)である$")]
fn start_elements(world: &mut QaWorld, phrase: String) {
    explore(&mut world.scene);
    let words = phrase.split('"');
    for (kind, name) in words.clone().step_by(2).zip(words.skip(1).step_by(2)) {
        let (spec, value) = element(kind, name);
        let scene = &mut world.scene;
        scene.start = scene.start.clone().with(spec);
        scene.inputs.extend(value);
    }
    drop(phrase);
}

#[then(expr = "判断モデルに問い合わせた操作種別の選択肢に input が含まれるかは {word} である")]
async fn input_offered(world: &mut QaWorld, offered: String) {
    world.run().await;
    let has = operation_options(world).contains(&"input".to_owned());
    assert_eq!(has, offered == "含む", "input among the options");
}

#[then(expr = "判断モデルに問い合わせた操作種別の選択肢に enter は含まれない")]
async fn enter_not_offered(world: &mut QaWorld) {
    world.run().await;
    let has = operation_options(world).contains(&"enter".to_owned());
    assert!(!has, "no enter among the options");
}

#[then(expr = "原因 {word} の skipped が記録されている")]
async fn skipped_recorded(world: &mut QaWorld, cause: String) {
    world.run().await;
    let skipped = |line: &Value| line.get("kind").and_then(Value::as_str) == Some("skipped");
    let causes: Vec<&str> = world
        .ran()
        .store
        .lines
        .iter()
        .filter(|line| skipped(line))
        .filter_map(|line| line.get("cause").and_then(Value::as_str))
        .collect();
    assert_eq!(causes, [cause.as_str()], "the skipped lines");
}

#[then(expr = "skipped は記録されていない")]
async fn no_skipped(world: &mut QaWorld) {
    world.run().await;
    let kinds = world.kinds();
    assert!(!kinds.contains(&"skipped".to_owned()), "no skipped line");
}

/// Whether the element `name` is among the explored, once the run is over.
async fn counted(world: &mut QaWorld, name: &str) -> bool {
    world.run().await;
    world.explored().contains(name)
}

#[then(expr = "{string} は探索済みの要素に数えられている")]
async fn is_counted(world: &mut QaWorld, name: String) {
    assert!(counted(world, &name).await, "{name} is explored");
}

#[then(expr = "{string} は探索済みの要素に数えられていない")]
async fn is_not_counted(world: &mut QaWorld, name: String) {
    assert!(!counted(world, &name).await, "{name} is not explored");
}
