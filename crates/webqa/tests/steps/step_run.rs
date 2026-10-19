//! Steps of the rule on one step, part: what the browser did.

use std::collections::BTreeSet;

use cucumber::{given, then};

use crate::judge::{Reply, Spoken};
use crate::page::{ElementSpec, PageSpec};
use crate::world::QaWorld;

/// The role of the target of an operation, when it is not a button.
const ROLES: [(&str, &str); 1] = [("input", "textbox")];

/// The operation a reply names, if it answers one.
const fn operation_of(reply: &Reply) -> Option<&str> {
    match reply {
        Reply::Answer(Spoken {
            op: Some((op, _)), ..
        }) => Some(op.as_str()),
        Reply::Answer(_) | Reply::Fail(_) => None,
    }
}

#[given(expr = "{string} は禁止要素に分類されておらず、実行の直前にも存在する")]
fn present(world: &mut QaWorld, name: String) {
    let said = world.scene.replies.back().and_then(operation_of);
    let role = ROLES.iter().find(|(op, _)| Some(*op) == said);
    let element = ElementSpec::new(role.map_or("button", |(_, role)| role), &name);
    world.scene.start.elements.push(element);
    drop(name);
}

#[then(expr = "ブラウザで操作 {string} を実行している")]
async fn executed(world: &mut QaWorld, op: String) {
    world.run().await;
    let target = ["click", "input"].contains(&op.as_str());
    let named = target.then_some("保存".to_owned());
    assert_eq!(world.touched(), [(op, named)], "the one operation");
}

#[then(expr = "探索済みの要素は {string} だけ である")]
async fn explored_only(world: &mut QaWorld, name: String) {
    world.run().await;
    assert_eq!(world.explored(), BTreeSet::from([name]), "only the target");
}

#[then(expr = "探索済みの要素は 空 である")]
async fn explored_nothing(world: &mut QaWorld) {
    world.run().await;
    assert_eq!(world.explored(), BTreeSet::new(), "nothing is explored");
}

#[given(expr = "直前のステップで {string} をクリックしたが画面が変化しなかった")]
fn clicked_unchanged(world: &mut QaWorld, name: String) {
    let scene = &mut world.scene;
    scene.start.elements.push(PageSpec::button(&name));
    scene.start.elements.push(PageSpec::link("ヘルプ"));
    scene.replies.push_back(Reply::op("click", 0.9).on(name));
}

#[given(
    expr = "判断モデルが対象要素の候補として {string} を {float}、{string} を {float} の確信度で返す"
)]
fn candidates(world: &mut QaWorld, first: String, p: f64, second: String, q: f64) {
    let reply = Reply::op("click", 0.9).on_ranked(first, p);
    world.scene.replies.push_back(reply.on_ranked(second, q));
}

#[then(expr = "{string} はクリックしていない")]
async fn not_clicked(world: &mut QaWorld, name: String) {
    world.run().await;
    let touched = world.touched();
    let second = touched.get(1).and_then(|(_, target)| target.clone());
    assert_ne!(second, Some(name), "the second click is not on it");
    let both = [("click", Some("保存")), ("click", Some("ヘルプ"))];
    let seen: Vec<(&str, Option<&str>)> = touched
        .iter()
        .map(|(op, target)| (op.as_str(), target.as_deref()))
        .collect();
    assert_eq!(seen, both, "the clicks");
}

#[given(expr = "対象ページに操作可能な要素が1つも無い")]
fn no_elements(world: &mut QaWorld) {
    world.scene.start.elements.clear();
}

#[then(expr = "判断モデルに送った画面状態の操作可能要素の一覧は空である")]
async fn nothing_listed(world: &mut QaWorld) {
    world.run().await;
    let text = world.ran().judge.texts.first().cloned();
    assert_eq!(text, Some(String::new()), "the list is empty");
}

#[then(expr = "ブラウザで {string} を実行している")]
async fn did(world: &mut QaWorld, op: String) {
    world.run().await;
    let touched = world.touched();
    assert!(
        touched.contains(&(op, None)),
        "the browser did it: {touched:?}"
    );
}
