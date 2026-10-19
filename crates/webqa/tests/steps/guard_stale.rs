//! Steps of the rule on the guards, part: the element changed before the operation.

use cucumber::{given, then, when};
use webqa_core::screen::ElementId;

use crate::judge::Reply;
use crate::page::PageSpec;
use crate::scene::found;
use crate::world::QaWorld;

/// The judge proposes `op` on the button `name`, next to a link "ヘルプ".
fn proposes(world: &mut QaWorld, op: &str, name: &str) {
    let scene = &mut world.scene;
    scene.start.elements.push(PageSpec::button(name));
    scene.start.elements.push(PageSpec::link("ヘルプ"));
    scene.replies.push_back(Reply::op(op, 0.9).on(name));
}

#[given(expr = "判断モデルが操作 {string} と対象要素 {string} を返す")]
fn proposes_target(world: &mut QaWorld, op: String, name: String) {
    proposes(world, &op, &name);
    drop((name, op));
}

#[given(expr = "判断モデルが操作 {string} とボタン {string} を返す")]
fn proposes_button(world: &mut QaWorld, op: String, name: String) {
    proposes(world, &op, &name);
    drop((name, op));
}

/// The next observations: the page as it was when the judge was asked, and
/// then `page`, as the browser shows it just before the operation.
fn then_shows(world: &mut QaWorld, page: PageSpec) {
    let asked = world.scene.start.clone();
    world.scene.looks.extend([asked, page]);
}

#[given(expr = "問い合わせから実行までの間に {string} がページから消える")]
fn gone(world: &mut QaWorld, name: String) {
    let mut page = world.scene.start.clone();
    page.elements.retain(|element| element.name != name);
    drop(name);
    then_shows(world, page);
}

#[given(expr = "問い合わせから実行までの間に、同じ位置の {string} がボタンからリンクに差し替わる")]
fn swapped_for_link(world: &mut QaWorld, name: String) {
    let mut page = world.scene.start.clone();
    let same = page
        .elements
        .iter_mut()
        .filter(|element| element.name == name);
    same.for_each(|element| *element = PageSpec::link(&name).node("e9"));
    drop(name);
    then_shows(world, page);
}

#[then(expr = "ブラウザで何もクリックしていない")]
async fn clicked_nothing(world: &mut QaWorld) {
    world.run().await;
    let touched = world.touched();
    assert!(touched.iter().all(|(op, _)| op != "click"), "{touched:?}");
}

#[given(expr = "対象ページの上下に {string} という名前のボタンが2つある")]
fn two_buttons(world: &mut QaWorld, name: String) {
    let elements = &mut world.scene.start.elements;
    elements.push(PageSpec::button(&name).node("e1"));
    elements.push(PageSpec::button(&name).node("e2"));
    drop(name);
}

#[given(expr = "判断モデルが操作 {string} と上の {string} を返す")]
fn proposes_upper(world: &mut QaWorld, op: String, name: String) {
    let reply = Reply::op(op, 0.9).on_nth(name, 0);
    world.scene.replies.push_back(reply);
}

#[when(expr = "問い合わせから実行までの間に2つの {string} の位置が入れ替わる")]
async fn exchanged(world: &mut QaWorld, name: String) {
    let mut page = world.scene.start.clone();
    let nodes = ["e2", "e1"].map(str::to_owned);
    let same = page
        .elements
        .iter_mut()
        .filter(|element| element.name == name);
    let each = same.zip(nodes);
    each.for_each(|(element, node)| element.node = Some(node));
    then_shows(world, page);
    world.run().await;
}

#[then(expr = "ブラウザで上の {string} をクリックしている")]
async fn clicked_upper(world: &mut QaWorld, name: String) {
    world.run().await;
    let touches = world.ran().browser.touches.iter();
    let mut clicks = touches.filter(|touch| touch.op == "click");
    let first = found(clicks.next(), "the browser clicked");
    let upper = ElementId {
        role: "button".to_owned(),
        name,
        nth: 0,
    };
    assert_eq!(first.target, Some(upper), "the first button of the name");
}
