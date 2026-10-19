//! The Given and When steps of the stages of a scenario, and the helpers
//! the steps of the recovery use too.

use cucumber::{given, when};
use webqa_core::config::raw::{DoneWhen, Element, Stage};
use webqa_core::event::Mode;

use crate::judge::Reply;
use crate::page::PageSpec;
use crate::scene::{Scene, found};
use crate::world::QaWorld;

/// The stages the feature names: (name, goal, the text that completes it).
const STAGES: [(&str, &str, &str); 2] = [
    ("ログイン", "ログインする", "ログインしました"),
    ("投稿", "投稿する", "投稿しました"),
];

/// The known stage `name`: its goal and the text that completes it.
fn known(name: &str) -> (&'static str, &'static str, &'static str) {
    found(
        STAGES.iter().find(|s| s.0 == name).copied(),
        "a known stage",
    )
}

/// The text of a page that completes the stage `name`.
fn sentence(name: &str) -> &'static str {
    known(name).2
}

/// The stage `name` with its default `done_when`.
fn stage(name: &str) -> Stage {
    let (_, goal, done) = known(name);
    Stage {
        name: name.to_owned(),
        goal: goal.to_owned(),
        done_when: DoneWhen {
            url: None,
            element: None,
            text: Some(done.to_owned()),
        },
    }
}

/// The position of the defined stage `name`, from 0.
pub fn index_of(world: &QaWorld, name: &str) -> usize {
    let at = world.scene.stages.iter().position(|s| s.name == name);
    found(at, "the scenario defines the stage")
}

/// Makes the scene a scenario of the stages `names`, in that order.
fn define(world: &mut QaWorld, names: &[&str]) {
    world.scene.mode = Mode::Scenario;
    world.scene.stages = names.iter().map(|name| stage(name)).collect();
}

/// The page that follows operation number `index` (from 0). The ones before
/// it, if not set yet, are copies of the start page.
pub fn after_at(scene: &mut Scene, index: usize) -> &mut PageSpec {
    while scene.after.len() <= index {
        scene.after.push_back(scene.start.clone());
    }
    found(scene.after.get_mut(index), "the page was added")
}

/// Adds `sentence` to the text of `page`, on a line of its own.
pub fn add_text(page: &mut PageSpec, sentence: &str) {
    let text = page.text.take();
    let joined = text.map_or_else(|| sentence.to_owned(), |t| format!("{t}\n{sentence}"));
    page.text = Some(joined);
}

/// Makes the page after the last queued operation (the start page when none
/// is queued) show `sentence`.
pub fn show_after_last(scene: &mut Scene, sentence: &str) {
    match scene.replies.len().checked_sub(1) {
        Some(last) => add_text(after_at(scene, last), sentence),
        None => add_text(&mut scene.start, sentence),
    }
}

#[given(expr = "決め打ちシナリオに段階 {string} と {string} がこの順に定義されている")]
#[given(expr = "決め打ちシナリオに段階 {string} と {string} がこの順に が定義されている")]
fn two_stages(world: &mut QaWorld, first: String, second: String) {
    define(world, &[&first, &second]);
    drop((first, second));
}

#[given(expr = "決め打ちシナリオに段階 {string} だけ が定義されている")]
fn one_stage(world: &mut QaWorld, only: String) {
    define(world, &[&only]);
    drop(only);
}

/// The stages whose texts a page shows when it is in the stage: the ones
/// before it too.
const DONE: [(&str, &[&str]); 2] = [("ログイン", &["ログイン"]), ("投稿", &["ログイン", "投稿"])];

#[given(expr = "段階 {string} にいて、その done_when 条件が満たされている")]
fn in_stage_done(world: &mut QaWorld, name: String) {
    let page = &mut world.scene.start;
    let done = found(
        DONE.iter().find(|(stage, _)| *stage == name),
        "a known stage",
    );
    for done in done.1 {
        add_text(page, sentence(done));
    }
    drop(name);
}

#[given(expr = "段階 {string} の done_when 条件は満たされていない")]
#[given(expr = "段階 {string} にいて、その done_when 条件は満たされていない")]
#[given(expr = "段階 {string} があれば、その done_when 条件は満たされていない")]
#[given(expr = "表示領域はページの一番下にある")]
const fn nothing_to_do(_world: &mut QaWorld) {}

#[given(expr = "段階 {string} の done_when 条件は満たされている")]
#[given(expr = "段階 {string} の done_when 条件も満たされている")]
fn later_done(world: &mut QaWorld, name: String) {
    add_text(&mut world.scene.start, sentence(&name));
    drop(name);
}

#[given(expr = "対象ページを開いた時点で、段階 {string} の done_when 条件が満たされている")]
fn done_at_open(world: &mut QaWorld, name: String) {
    add_text(&mut world.scene.start, sentence(&name));
    drop(name);
}

#[when(expr = "実行を開始する")]
#[when(expr = "段階 {string} から段階 {string} に進む")]
#[when(expr = "1ステップを実行して段階 {string} に進む")]
async fn run(world: &mut QaWorld) {
    world.run().await;
}

#[given(
    expr = "段階 {string} で、画面が変化せず探索済みの要素も増えないステップが {int} 回続いている"
)]
fn unchanged_in_stage(world: &mut QaWorld, name: String, times: usize) {
    let waits = (0..times).map(|_| Reply::op("wait", 0.9));
    world.scene.replies.extend(waits);
    drop(name);
}

#[given(expr = "段階 {string} の done_when 条件が満たされている")]
fn done_after_last(world: &mut QaWorld, name: String) {
    let scene = &mut world.scene;
    let called: Vec<u64> = scene.marks.get("called").copied().into_iter().collect();
    for called in called {
        let queued = scene.replies.len();
        let keep = usize::try_from(called).unwrap_or(queued);
        drop(scene.replies.drain(..queued.saturating_sub(keep)));
    }
    show_after_last(scene, sentence(&name));
    drop(name);
}

#[given(
    expr = "判断モデルの呼び出し上限を {int} 回に設定し、段階 {string} で {int} 回呼び出し済みである"
)]
fn budget_spent(world: &mut QaWorld, cap: u32, name: String, called: usize) {
    drop(name);
    let scene = &mut world.scene;
    scene.limits.insert("max_decisions", cap);
    scene
        .replies
        .extend((0..called).map(|_| Reply::op("wait", 0.9)));
    scene
        .marks
        .insert("called", u64::try_from(called).unwrap_or(0));
}

/// Makes the `done_when` of the stage `name` the URL path `path` (if any)
/// and the heading `heading`.
fn declare(world: &mut QaWorld, name: &str, path: Option<String>, heading: &str) {
    let at = index_of(world, name);
    let element = Element {
        role: "heading".to_owned(),
        name: heading.to_owned(),
    };
    if let Some(stage) = world.scene.stages.get_mut(at) {
        stage.done_when = DoneWhen {
            url: path,
            element: Some(element),
            text: None,
        };
    }
}

#[given(
    expr = "段階 {string} の done_when に、URL のパス {string} と見出し {string} の2つの宣言がある"
)]
fn url_and_heading(world: &mut QaWorld, name: String, path: String, heading: String) {
    declare(world, &name, Some(path), &heading);
    drop((name, heading));
}

#[given(expr = "段階 {string} の done_when に、見出し {string} の宣言だけがある")]
fn heading_only(world: &mut QaWorld, name: String, heading: String) {
    declare(world, &name, None, &heading);
    drop((name, heading));
}

#[given(
    expr = "段階 {string} にいて、今の URL のパスは {string} で、見出し {string} はページにある"
)]
fn path_with_heading(world: &mut QaWorld, name: String, path: String, heading: String) {
    world.scene.start.path = path;
    world.scene.start.headings.push((heading, false));
    drop(name);
}

#[given(
    expr = "段階 {string} にいて、今の URL のパスは {string} で、見出し {string} はページに無い"
)]
fn path_without_heading(world: &mut QaWorld, name: String, path: String, heading: String) {
    world.scene.start.path = path;
    drop((name, heading));
}

#[given(expr = "段階 {string} にいて、見出し {string} はページにあるが表示領域より下にある")]
fn heading_below(world: &mut QaWorld, name: String, heading: String) {
    world.scene.start.headings.push((heading, true));
    drop(name);
}
