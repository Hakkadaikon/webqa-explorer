//! Steps of the rule on the guards, part: the forbidden elements.

use cucumber::{given, then};

use crate::page::{ElementSpec, PageSpec};
use crate::world::QaWorld;

/// The page shows `element` and a button "ヘルプ" next to it.
fn shows(world: &mut QaWorld, element: ElementSpec) {
    let elements = &mut world.scene.start.elements;
    elements.push(element);
    elements.push(PageSpec::button("ヘルプ"));
}

#[given(expr = "対象ページに ボタン {string} がある")]
fn has_button(world: &mut QaWorld, name: String) {
    shows(world, PageSpec::button(&name));
    drop(name);
}

#[given(expr = "対象ページに 入力欄 {string}\\(お支払いフォーム内\\) がある")]
fn has_card_field(world: &mut QaWorld, name: String) {
    shows(
        world,
        PageSpec::textbox(&name).form_action("/checkout/payment"),
    );
    drop(name);
}

#[given(expr = "対象ページに 代替テキスト {string} の画像ボタン がある")]
fn has_image_button(world: &mut QaWorld, alt: String) {
    shows(world, PageSpec::button(&alt));
    drop(alt);
}

#[given(expr = "対象ページに action が {string} のフォームの送信ボタン {string} がある")]
fn has_submit(world: &mut QaWorld, action: String, name: String) {
    shows(world, PageSpec::button(&name).form_action(&action));
    drop((action, name));
}

#[given(expr = "対象ページに 遷移先が {string} のリンク {string} がある")]
fn has_link_to(world: &mut QaWorld, to: String, name: String) {
    shows(world, PageSpec::link(&name).href(&to));
    drop((name, to));
}

#[given(expr = "対象ページに 設定の禁止リストに一致するボタン {string} がある")]
fn has_listed_button(world: &mut QaWorld, name: String) {
    shows(world, PageSpec::button(&name).forbidden_listed());
    drop(name);
}

#[then(expr = "ブラウザで {string} を操作していない")]
async fn did_not_operate(world: &mut QaWorld, name: String) {
    world.run().await;
    let touched = world.touched();
    assert!(
        touched.iter().all(|(_, t)| t.as_ref() != Some(&name)),
        "{touched:?}"
    );
}

#[then(expr = "{string} は探索済みの要素に含まれない")]
async fn not_explored(world: &mut QaWorld, name: String) {
    world.run().await;
    assert!(!world.explored().contains(&name), "it is not explored");
}

#[given(expr = "対象ページに 許可リストにも一致するが名前が {string} を含むボタン {string} がある")]
fn has_allowed_button(world: &mut QaWorld, keyword: String, name: String) {
    assert!(name.contains(&keyword), "the name has the keyword");
    world
        .scene
        .start
        .elements
        .push(PageSpec::button(&name).allowed_listed());
    drop((keyword, name));
}
