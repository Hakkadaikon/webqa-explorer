//! Steps of the rule on the load, part: the accesses the window counts.

use cucumber::{given, then, when};
use serde_json::json;

use crate::judge::Reply;
use crate::page::PageSpec;
use crate::scene::found;
use crate::shared::{OTHER, first_wait, foreign_page};
use crate::world::QaWorld;

/// The link the judge follows out of the allowed origin.
const OUTSIDE: &str = "外部の記事";

#[when(expr = "固まりから復旧するために対象ページを開き直す")]
async fn reload(world: &mut QaWorld) {
    world.scene.limits.insert("freeze_threshold", 1);
    world.run().await;
}

/// The judge's last reply clicks the link out of the allowed origin, which
/// leads to a page of another origin.
fn go_outside(world: &mut QaWorld) {
    let scene = &mut world.scene;
    scene.replies.pop_back();
    scene.replies.push_back(Reply::op("click", 0.9).on(OUTSIDE));
    scene.start.elements.push(PageSpec::link(OUTSIDE));
}

#[when(expr = "許可ドメインの外から許可ドメインのページへ戻る")]
async fn return_inside(world: &mut QaWorld) {
    go_outside(world);
    world.scene.after.push_back(foreign_page(OTHER));
    world.run().await;
}

#[when(expr = "課金画面から既定の離脱操作をする")]
async fn leave_the_paywall(world: &mut QaWorld) {
    let pages = [PageSpec::at("/pricing"), PageSpec::at("/")];
    world.scene.after.extend(pages);
    world.run().await;
}

#[then(expr = "時間窓の操作回数は {int} 回になっている")]
async fn window_count(world: &mut QaWorld, used: usize) {
    world.run().await;
    let kinds = world.kinds();
    let wait = first_wait(&kinds);
    let before = kinds.iter().take(wait);
    assert_eq!(
        before.filter(|kind| *kind == "executed").count(),
        used,
        "executed"
    );
}

#[then(expr = "時間窓が明ける前のステップでは判断モデルに問い合わせない")]
async fn no_question_before_the_window_ends(world: &mut QaWorld) {
    world.run().await;
    let kinds = world.kinds();
    let wait = first_wait(&kinds);
    let before = kinds.iter().take(wait);
    let asked = before.filter(|kind| *kind == "asked").count();
    assert_eq!(asked, world.scene.replies.len(), "only the given questions");
}

#[given(expr = "固まりを検出している")]
fn frozen(world: &mut QaWorld) {
    world.scene.limits.insert("freeze_threshold", 2);
}

#[given(expr = "許可ドメインの外にいる")]
fn outside(world: &mut QaWorld) {
    go_outside(world);
    let home = PageSpec::at("/").with(PageSpec::link(OUTSIDE));
    let pages = [home, foreign_page(OTHER)];
    world.scene.after.extend(pages);
}

#[given(expr = "課金画面が表示されている")]
fn paywall_shown(world: &mut QaWorld) {
    let pages = [
        PageSpec::at("/"),
        PageSpec::at("/pricing"),
        PageSpec::at("/"),
    ];
    world.scene.after.extend(pages);
}

/// The core's first operation of its own after the wait was `op` for `cause`.
fn recovered_after_waiting(world: &QaWorld, op: &str, cause: &str) {
    let lines = world.lines();
    let kinds = world.kinds();
    let wait = first_wait(&kinds);
    let later = lines
        .iter()
        .skip(wait)
        .filter(|line| line.get("kind") == Some(&json!("recovered")));
    let first = found(later.take(1), "the core operated after the wait");
    assert_eq!(
        (first.get("op"), first.get("cause")),
        (Some(&json!(op)), Some(&json!(cause))),
        "what the core operated, and why"
    );
}

#[then(expr = "時間窓が明けたあとのステップで ページの開き直し をしている")]
async fn reloaded_after_the_window(world: &mut QaWorld) {
    world.run().await;
    recovered_after_waiting(world, "reload", "frozen");
}

#[then(expr = "時間窓が明けたあとのステップで 許可ドメインへの戻り をしている")]
async fn returned_after_the_window(world: &mut QaWorld) {
    world.run().await;
    recovered_after_waiting(world, "return", "outside");
}

#[then(expr = "時間窓が明けたあとのステップで 既定の離脱操作 をしている")]
async fn left_after_the_window(world: &mut QaWorld) {
    world.run().await;
    recovered_after_waiting(world, "back", "paywall");
}
