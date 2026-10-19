//! Steps of the Rule on memory: what counts as the same screen.

use std::collections::BTreeSet;

use cucumber::{given, then, when};
use webqa_core::event::Mode;
use webqa_core::explore::Memory;
use webqa_core::redact::Redactor;

use super::{button, link, path_of, paths, routes, saved, screen_count, text_at};
use crate::judge::Reply;
use crate::page::PageSpec;
use crate::world::QaWorld;

/// The element the memory of the other target holds.
const OTHER: &str = "別の対象の画面";

/// Whether `shown` is `wanted`.
fn is(shown: impl Into<String>, wanted: impl Into<String>) -> bool {
    shown.into() == wanted.into()
}

#[given(expr = "対象 {string} の探索の記憶がある")]
fn other_memory(world: &mut QaWorld, origin: String) {
    let page = PageSpec::at("/").with(button(OTHER)).build(&origin);
    let mut memory = Memory::default();
    let screen = Redactor::default().screen(page.screen);
    let key = memory.visit(&screen, &BTreeSet::new());
    assert_eq!(memory.screen_count(), 1, "the memory has {key:?}");
    let json = memory.to_json(&origin).to_string();
    world.scene.memories.insert(origin, json);
}

#[when(expr = "対象 {string} で新しい実行を始める")]
async fn start_on(world: &mut QaWorld, origin: String) {
    assert!(
        is(world.scene.origin(), origin),
        "the run is on this target"
    );
    world.run().await;
}

#[then(expr = "探索の記憶に {string} の画面は含まれない")]
async fn no_other_screen(world: &mut QaWorld, origin: String) {
    world.run().await;
    let saved = saved(world);
    assert!(
        !is(text_at(&saved, &["origin"]), origin),
        "the memory is of this target"
    );
    assert!(
        !saved.to_string().contains(OTHER),
        "the other screen is not in it"
    );
}

#[given(expr = "画面 {string} を表示していて、表示領域より下にリンク {string} がある")]
fn link_below(world: &mut QaWorld, screen: String, name: String) {
    let scene = &mut world.scene;
    scene.mode = Mode::Explore;
    let page = PageSpec::at(path_of(screen)).with(button("保存"));
    scene.start = page.with(link(name).below());
}

/// The page with the elements named `name` in the viewport.
fn revealed(mut page: PageSpec, name: impl Into<String>) -> PageSpec {
    let name = name.into();
    let named = page.elements.iter_mut().filter(|e| e.name == name);
    named.for_each(|element| element.below = false);
    page
}

#[when(expr = "操作 {string} を実行し、表示領域に {string} が現れる")]
async fn scrolls_into_view(world: &mut QaWorld, op: String, name: String) {
    let page = revealed(world.scene.start.clone(), name);
    world.scene.after.push_back(page);
    world.scene.replies.push_back(Reply::op(op, 0.9));
    world.run().await;
}

#[then(expr = "今の画面は探索の記憶の {string} のままである")]
async fn same_screen(world: &mut QaWorld, screen: String) {
    world.run().await;
    assert_eq!(paths(world), [path_of(screen)], "the one screen");
}

#[then(expr = "探索の記憶に新しい画面は加わっていない")]
async fn no_new_screen(world: &mut QaWorld) {
    world.run().await;
    assert_eq!(screen_count(world), 1, "the one screen of before");
    assert_eq!(routes(world), [], "no edge");
}

#[given(expr = "画面 {string} を表示している")]
fn shows(world: &mut QaWorld, screen: String) {
    world.scene.mode = Mode::Explore;
    world.scene.start = PageSpec::at(path_of(screen)).with(button("保存"));
}

#[when(expr = "ボタン {string} をクリックし、ページ全体の操作可能要素が変わる")]
async fn click_changes(world: &mut QaWorld, name: String) {
    world.scene.show_button(&name);
    let changed = PageSpec::at(&world.scene.start.path).with(button("通知"));
    world.scene.after.push_back(changed);
    world
        .scene
        .replies
        .push_back(Reply::op("click", 0.9).on(name));
    world.run().await;
}

#[then(expr = "探索の記憶に新しい画面が加わり、{string} からその画面への遷移が記録されている")]
async fn new_screen(world: &mut QaWorld, screen: String) {
    world.run().await;
    let path = path_of(screen).to_owned();
    assert_eq!(screen_count(world), 2, "the page is a new screen");
    assert_eq!(
        routes(world),
        [(path.clone(), path)],
        "the click leads to it"
    );
}
