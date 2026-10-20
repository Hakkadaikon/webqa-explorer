//! Steps of the rule on one step's question and run: what the judge was sent.

use cucumber::gherkin::Step;
use cucumber::{given, then};
use serde_json::{Value, json, to_value};
use webqa_core::event::Mode;

use crate::page::PageSpec;
use crate::scene::found;
use crate::shared::{login_only, url_done};
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "判断モデルに送る画面状態はアクセシビリティ情報のテキストで、操作可能要素の一覧を含む",
    "1ステップで操作種別・対象要素・エラー表示の有無・ゴール達成の有無を問い合わせる",
    "画面が変わらないのに同じ要素を選んだら、次に確信度の高い候補へ切り替える",
    "操作可能な要素が無いページでも問い合わせ、要素を対象にしない操作だけを実行する",
    "判断モデルに送る操作可能要素の一覧は、表示領域の中の要素だけにする",
    "ガードレールを通った操作はそのままブラウザで実行する",
    "判断モデルの応答が使えないときは操作せず、応答の異常として記録して次のステップへ進む",
];

/// The names of the elements in the screen text the judge was sent, whose
/// lines read `n: role "name"`.
fn listed(world: &QaWorld) -> Vec<String> {
    let text = found(world.ran().judge.texts.first(), "the judge was asked");
    let quoted = text.lines().filter_map(|line| line.split_once(" \""));
    quoted
        .map(|(_, name)| name.trim_end_matches('"').to_owned())
        .collect()
}

#[given(expr = "対象ページに操作可能なボタン {string} とリンク {string} がある")]
fn button_and_link(world: &mut QaWorld, button: String, link: String) {
    let elements = &mut world.scene.start.elements;
    elements.push(PageSpec::button(&button));
    elements.push(PageSpec::link(&link));
    drop((button, link));
}

#[then(
    expr = "判断モデルに送った画面状態はスクリーンショットではなくアクセシビリティ情報のテキストである"
)]
async fn text_of_roles_and_names(world: &mut QaWorld) {
    world.run().await;
    let text = world.ran().judge.texts.first().cloned();
    let expected = "1: button \"保存\"\n2: link \"ヘルプ\"".to_owned();
    assert_eq!(
        text,
        Some(expected),
        "the screen text of the first question"
    );
}

#[then(expr = "その画面状態には {string} と {string} が操作可能要素として含まれる")]
async fn both_listed(world: &mut QaWorld, first: String, second: String) {
    world.run().await;
    assert_eq!(listed(world), [first, second], "the listed elements");
}

#[given(
    expr = "対象ページの表示領域にボタン {string} があり、表示領域より下にリンク {string} がある"
)]
fn below_the_viewport(world: &mut QaWorld, button: String, link: String) {
    let elements = &mut world.scene.start.elements;
    elements.push(PageSpec::button(&button));
    elements.push(PageSpec::link(&link).below());
    drop((button, link));
}

#[then(expr = "判断モデルに送った画面状態の操作可能要素の一覧に {string} が含まれる")]
async fn listed_with(world: &mut QaWorld, name: String) {
    world.run().await;
    assert!(listed(world).contains(&name), "the list has it");
}

#[then(expr = "その一覧に {string} は含まれない")]
async fn listed_without(world: &mut QaWorld, name: String) {
    world.run().await;
    assert!(!listed(world).contains(&name), "the list lacks it");
}

#[given(expr = "決め打ちシナリオを実行している")]
fn scripted(world: &mut QaWorld) {
    world.scene.mode = Mode::Scenario;
    world.scene.stages = login_only(url_done("/home"));
}

/// The name an item of the table has as a question, and the kind of the
/// question.
const ITEMS: [(&str, &str, &str); 4] = [
    ("操作種別", "operation", "choice"),
    ("対象要素", "target", "choice"),
    ("エラー表示の有無", "error_shown", "noul"),
    ("ゴール達成の有無", "goal_reached", "noul"),
];

/// The operation kinds the judge may choose from.
const OPERATIONS: [&str; 9] = [
    "click",
    "input",
    "scroll_down",
    "scroll_up",
    "back",
    "enter",
    "wait",
    "done",
    "stuck",
];

/// The first column of the table, without its heading.
fn table_items(step: &Step) -> Vec<Option<&str>> {
    let table = found(step.table.as_ref(), "the step has a table");
    let rows = table.rows.iter().skip(1);
    rows.map(|row| row.first().map(String::as_str)).collect()
}

/// The field `name` of each question of the first call, as JSON.
fn question_field(world: &QaWorld, name: &str) -> Vec<Option<Value>> {
    let first = world.ran().judge.questions.first();
    let questions = found(first, "the judge was asked");
    let seen = questions.iter().filter_map(|q| to_value(q).ok());
    seen.map(|question| question.get(name).cloned()).collect()
}

#[then(expr = "判断モデルに次の4項目を問い合わせている")]
async fn four_items(world: &mut QaWorld, step: &Step) {
    world.run().await;
    let items = ITEMS.map(|(item, _, _)| Some(item));
    assert_eq!(table_items(step), items, "the table lists the four items");
    let ids = ITEMS.map(|(_, id, _)| Some(json!(id)));
    let kinds = ITEMS.map(|(_, _, kind)| Some(json!(kind)));
    assert_eq!(question_field(world, "id"), ids, "the questions");
    assert_eq!(question_field(world, "kind"), kinds, "the kinds");
    let options = question_field(world, "options").into_iter().next();
    let operations = Some(Some(json!(OPERATIONS)));
    assert_eq!(options, operations, "the operation kinds");
}
