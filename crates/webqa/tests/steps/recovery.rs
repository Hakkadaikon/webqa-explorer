//! Steps of one Rule. The scenarios whose steps are all here are named in
//! `READY`.

use cucumber::{given, then, when};
use serde_json::Value;

use crate::judge::Reply;
use crate::page::PageSpec;
use crate::remembered::remember;
use crate::stages_given::after_at;
use crate::world::{Beat, QaWorld};

/// The lines of the log of this `kind`.
pub fn lines_of(world: &QaWorld, kind: &str) -> Vec<Value> {
    let is_kind = |line: &&Value| line.get("kind").and_then(Value::as_str) == Some(kind);
    world.lines().iter().filter(is_kind).cloned().collect()
}

/// The text of the field `field` of the line, if it is a string.
pub fn field<'a>(line: &'a Value, field: &str) -> Option<&'a str> {
    line.get(field).and_then(Value::as_str)
}

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "画面が3回続けて変化しなければ、ページを開き直して探索を続け、探索の記憶は残す",
    "固まりを検出しているステップでは判断モデルに問い合わせない",
    "ページが応答しないステップが3回続けば、画面が変化しないステップと同じく開き直す",
    "画面が変化すれば固まりの数え直しになる",
    "スクロールで表示領域の要素が変われば、画面が変化したと数えて固まりの数え直しになる",
    "ページの一番下でスクロールして表示領域の要素が変わらなければ、画面が変化しないステップと数える",
    "操作の結果ドメイン外に出たら記録して許可ドメインへ戻り、ドメイン外では問い合わせない",
    "許可ドメインかどうかは、最終的な遷移先のオリジンの完全一致で判定する",
    "リダイレクトされたら、最終的な遷移先で許可ドメインかどうかを判定する",
];

#[given(expr = "探索済みの要素は {string} と {string} である")]
fn explored_two(world: &mut QaWorld, first: String, second: String) {
    let others = ["操作1", "操作2", "操作3"];
    let mut shown = vec![("button", first.as_str()), ("link", second.as_str())];
    shown.extend(others.iter().map(|name| ("button", *name)));
    let screens = [("/", shown.as_slice())];
    remember(
        &mut world.scene,
        &screens,
        &[(0, &first, 0), (0, &second, 0)],
    );
    let start = PageSpec::at("/")
        .with(PageSpec::button(&first))
        .with(PageSpec::link(&second));
    world.scene.start = others
        .iter()
        .fold(start, |page, name| page.with(PageSpec::button(name)));
    drop((first, second));
}

#[given(expr = "行き詰まりは検出されていない")]
#[given(expr = "{string} をクリックしても、ページが設定の待ち時間内に応答しない")]
const fn nothing_to_set(_world: &mut QaWorld) {}

#[then(expr = "対象ページを初期状態から開き直している")]
async fn reopened(world: &mut QaWorld) {
    world.run().await;
    assert!(
        world.touched().contains(&("reload".to_owned(), None)),
        "the page was reopened"
    );
    let causes: Vec<_> = lines_of(world, "recovered")
        .iter()
        .filter_map(|line| field(line, "cause").map(str::to_owned))
        .collect();
    assert!(causes.contains(&"frozen".to_owned()), "frozen: {causes:?}");
}

#[then(expr = "その間は判断モデルに問い合わせていない")]
async fn not_asked_meanwhile(world: &mut QaWorld) {
    world.run().await;
    assert_eq!(world.when_beat(), Beat::Core, "the core operated");
}

#[then(expr = "探索済みの要素は {string} と {string} のままである")]
async fn explored_stay(world: &mut QaWorld, first: String, second: String) {
    world.run().await;
    let explored = world.explored();
    assert!(explored.contains(&first), "{first} is explored");
    assert!(explored.contains(&second), "{second} is explored");
}

#[given(expr = "判断モデルは毎回、操作 {string} と対象要素 {string} を返す")]
fn replies_each_time(world: &mut QaWorld, op: String, target: String) {
    let scene = &mut world.scene;
    let shown = after_at(scene, 0);
    shown.elements.push(PageSpec::link(&target));
    let replies = (0..3).map(|_| Reply::op(op.as_str(), 0.9).on(&target));
    scene.replies.extend(replies);
    drop((op, target));
}

#[when(expr = "そのステップが {int} 回続く")]
async fn steps_go_on(world: &mut QaWorld, times: usize) {
    world.run().await;
    assert!(times > 0, "the step goes on");
}

/// The operations the browser got, as (op, target) pairs.
fn touches(names: &[(&str, Option<&str>)]) -> Vec<(String, Option<String>)> {
    let to_owned =
        |(op, target): &(&str, Option<&str>)| ((*op).to_owned(), target.map(str::to_owned));
    names.iter().map(to_owned).collect()
}

#[then(expr = "{int} 回目と {int} 回目のあとは対象ページを開き直していない")]
async fn not_reopened_yet(world: &mut QaWorld, first: usize, second: usize) {
    assert_eq!((first, second), (1, 2), "the steps before the end");
    world.run().await;
    let expected = touches(&[
        ("click", Some("保存")),
        ("click", Some("ヘルプ")),
        ("click", Some("ヘルプ")),
        ("click", Some("ヘルプ")),
    ]);
    let touched = world.touched();
    assert_eq!(
        touched.get(..4),
        Some(expected.as_slice()),
        "before the end"
    );
}

#[then(expr = "{int} 回目のあとに対象ページを初期状態から開き直している")]
async fn reopened_after(world: &mut QaWorld, times: usize) {
    world.run().await;
    let touched = world.touched();
    let fifth = touched.get(times + 1).cloned();
    assert_eq!(
        fifth,
        Some(("reload".to_owned(), None)),
        "the fifth operation"
    );
}

#[then(expr = "探索済みの要素に {string} が残っている")]
async fn explored_remains(world: &mut QaWorld, name: String) {
    world.run().await;
    assert!(world.explored().contains(&name), "{name} remains");
}

/// The start page with `操作1` below the viewport.
fn scrolled(world: &QaWorld) -> PageSpec {
    let mut page = world.scene.start.clone();
    for element in &mut page.elements {
        element.below = element.name == "操作1" || element.below;
    }
    page
}

#[when(expr = "画面が変化する操作を1回実行する")]
async fn changing_step(world: &mut QaWorld) {
    let next = world.scene.replies.len();
    world.scene.replies.push_back(Reply::op("scroll_down", 0.9));
    *after_at(&mut world.scene, next) = scrolled(world);
    world.run().await;
}

#[given(expr = "スクロールすると、表示領域の中の操作可能要素が変わる")]
fn scroll_changes(world: &mut QaWorld) {
    let last = world.scene.replies.len().saturating_sub(1);
    *after_at(&mut world.scene, last) = scrolled(world);
}

#[then(expr = "その後、画面が変化しないステップが {int} 回続いてもページを開き直さない")]
async fn no_reload_after_change(world: &mut QaWorld, times: usize) {
    let waits = (0..times).map(|_| Reply::op("wait", 0.9));
    world.scene.replies.extend(waits);
    world.ran = None;
    world.run().await;
    let reloads = world.touched().into_iter().filter(|(op, _)| op == "reload");
    assert_eq!(reloads.count(), 0, "the page was not reopened");
}

#[given(
    expr = "判断モデルが操作 {string} を返し、スクロールしても表示領域の中の操作可能要素と URL は変わらない"
)]
fn scroll_stays(world: &mut QaWorld, op: String) {
    world.scene.replies.push_back(Reply::op(op, 0.9));
}
