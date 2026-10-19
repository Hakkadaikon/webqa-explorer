//! Steps of the Rule on memory: which element the judge is offered.

use cucumber::{given, then, when};
use webqa_core::event::Mode;

use super::view::{
    assert_moves, last_call, offered_names, remember_back, remember_pressed, settings_with,
};
use super::{MOVES, button, lacks, link, path_of};
use crate::judge::Reply;
use crate::page::PageSpec;
use crate::world::QaWorld;

#[given(expr = "対象ページに未操作の要素 {string} がある")]
fn unoperated(world: &mut QaWorld, name: String) {
    world.scene.start = world.scene.start.clone().with(link(name));
}

#[given(expr = "操作済みの要素 {string} の遷移先に未探索の分岐が残っている")]
fn branch_left(world: &mut QaWorld, name: String) {
    world.scene.show_button(&name);
    remember_pressed(world, name);
}

#[when(expr = "探索モードで次の操作を選ぶ")]
async fn choose(world: &mut QaWorld) {
    world.scene.mode = Mode::Explore;
    world.run().await;
}

#[then(expr = "{string} を選んでいる")]
async fn chosen(world: &mut QaWorld, name: String) {
    world.run().await;
    assert_eq!(offered_names(world, 0), [name], "the one candidate");
}

#[given(expr = "対象ページに未操作の要素は無い")]
fn nothing_unoperated(world: &mut QaWorld) {
    world.scene.start.elements.clear();
}

#[given(expr = "対象ページの要素はすべて探索し終えている")]
fn all_explored(world: &mut QaWorld) {
    settings_with(world, &["通知"]);
}

#[given(expr = "戻った先の画面 {string} に未操作の要素 {string} が残っている")]
fn left_behind(world: &mut QaWorld, top: String, name: String) {
    assert_eq!(path_of(top), "/", "the page left behind is the top page");
    remember_back(world, name);
}

#[then(expr = "選んだ操作は {string}、{string}、{string} のいずれかである")]
async fn chose_move(world: &mut QaWorld, first: String, second: String, third: String) {
    world.run().await;
    assert_eq!([first, second, third], MOVES.map(String::from), "the moves");
    assert_moves(world);
}

#[given(expr = "{string} を押すと画面 {string} へ遷移し、そこに未操作の要素 {string} がある")]
fn press_leads(world: &mut QaWorld, press: String, screen: String, name: String) {
    let scene = &mut world.scene;
    scene.mode = Mode::Explore;
    scene.start = PageSpec::at("/").with(button(press));
    let reached = PageSpec::at(path_of(screen)).with(button(name));
    scene.after.extend([reached, scene.start.clone()]);
}

#[when(expr = "{string} を押して画面 {string} へ戻る")]
async fn press_and_back(world: &mut QaWorld, press: String, screen: String) {
    assert_eq!(path_of(screen), "/", "the run comes back to the top page");
    let replies = [Reply::op("click", 0.9).on(press), Reply::op("back", 0.9)];
    world.scene.replies.extend(replies);
    world.run().await;
}

#[then(expr = "{string} は操作済みだが、まだ探索し終えた要素として扱われていない")]
async fn not_done_yet(world: &mut QaWorld, name: String) {
    world.run().await;
    assert!(world.explored().contains(&name), "it was operated");
    assert_eq!(
        offered_names(world, last_call(world)),
        [name],
        "its branch is left"
    );
}

#[given(expr = "対象ページの未操作の要素は {string} だけである")]
fn only_element(world: &mut QaWorld, name: String) {
    world.scene.start = PageSpec::at("/").with(button(name));
}

#[then(expr = "{string} を選んでいない")]
async fn not_chosen(world: &mut QaWorld, name: String) {
    world.run().await;
    assert!(
        lacks(&offered_names(world, 0), name),
        "it is not a candidate"
    );
}

#[given(expr = "対象ページの要素は {string} を除いてすべて探索し終えている")]
fn all_explored_but(world: &mut QaWorld, name: String) {
    settings_with(world, &["通知"]);
    world.scene.start = world.scene.start.clone().with(button(name));
}

#[then(
    expr = "{string} を選んでおらず、選んだ操作は {string}、{string}、{string} のいずれかである"
)]
async fn not_chosen_but_moves(world: &mut QaWorld, name: String, a: String, b: String, c: String) {
    world.run().await;
    assert_eq!([a, b, c], MOVES.map(String::from), "the moves");
    assert!(
        lacks(&offered_names(world, 0), name),
        "it is not a candidate"
    );
    assert_moves(world);
}
