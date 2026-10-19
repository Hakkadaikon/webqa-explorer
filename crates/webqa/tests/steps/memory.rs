//! Steps of the Rule on the memory of the exploration and its priorities.

use std::collections::BTreeSet;

use cucumber::{given, then, when};
use serde_json::Value;

use crate::page::{ElementSpec, PageSpec};
use crate::scene::found;
use crate::shared::{button, text_at};
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "探索の記憶を次の実行に引き継ぐ",
    "初めての実行では空の記憶から始める",
    "探索の記憶が壊れていたら、空の記憶で始めてそのことを記録する",
    "別の対象の探索の記憶は引き継がない",
    "未操作の要素を、未探索の分岐が残る操作済み要素より先に選ぶ",
    "未操作の要素が無ければ、未探索の分岐が残る操作済み要素を戻る・スクロールより先に選ぶ",
    "どの要素にも未探索の分岐が残っていなければ、戻るかスクロールを選ぶ",
    "遷移先を探索し終えるまで、途中のボタンを探索し終えた要素として扱わない",
    "探索モードでも決済導線の要素は選ばない",
    "禁止要素は未操作のまま残っていても、探索の候補に入れない",
    "スクロールしても、探索の記憶では同じ画面として扱う",
    "ページ全体の操作可能要素が変われば、探索の記憶では別の画面として扱う",
];

/// The screens of the feature, by name, and the paths that show them.
const PATHS: [(&str, &str); 2] = [("トップ", "/"), ("設定", "/settings")];

/// The operations offered when no element is: the moves.
const MOVES: [&str; 3] = ["back", "scroll_down", "scroll_up"];

/// The path of the screen the feature calls `name`.
fn path_of(name: impl Into<String>) -> &'static str {
    let name = name.into();
    let path = PATHS.iter().find(|(shown, _)| *shown == name);
    found(path.map(|(_, path)| *path), "the feature names the screen")
}

/// A link named `name`.
fn link(name: impl Into<String>) -> ElementSpec {
    PageSpec::link(&name.into())
}

/// The names of `list` without `name`: `name` is not in it.
fn lacks(list: &[String], name: impl Into<String>) -> bool {
    !list.contains(&name.into())
}

/// The memory the run saved for the first allowed origin.
fn saved(world: &QaWorld) -> Value {
    let text = world.ran().store.memories.get(&world.scene.origin());
    let text = found(text, "the run saved a memory");
    found(serde_json::from_str(text).ok(), "the memory is JSON")
}

/// How many screens the saved memory has.
fn screen_count(world: &QaWorld) -> usize {
    let saved = saved(world);
    let screens = saved.get("screens").and_then(Value::as_object);
    screens.map_or(0, serde_json::Map::len)
}

/// The paths of the screens of the saved memory.
fn paths(world: &QaWorld) -> Vec<String> {
    let saved = saved(world);
    let screens = saved.get("screens").and_then(Value::as_object);
    let screens = screens.into_iter().flat_map(|screens| screens.values());
    screens.map(|screen| text_at(screen, &["path"])).collect()
}

/// The edges of the saved memory as (path of from, path of to).
fn routes(world: &QaWorld) -> Vec<(String, String)> {
    let saved = saved(world);
    let edges = saved.get("edges").and_then(Value::as_array);
    let path = |edge: &Value, end: &str| {
        let key = text_at(edge, &[end]);
        text_at(&saved, &["screens", &key, "path"])
    };
    let edges = edges.into_iter().flatten();
    edges
        .map(|edge| (path(edge, "from"), path(edge, "to")))
        .collect()
}

#[path = "memory_select.rs"]
mod select;

#[path = "memory_view.rs"]
mod view;
use view::remember_pressed;

#[path = "memory_screens.rs"]
mod screens;

#[given(
    expr = "前回の実行で画面 {string} から {string} を押して画面 {string} へ遷移した記録がある"
)]
fn remembered(world: &mut QaWorld, from: String, pressed: String, to: String) {
    assert_eq!(path_of(to), "/settings", "the memory leads to the settings");
    world.scene.start = PageSpec::at(path_of(from)).with(button(&pressed));
    remember_pressed(world, pressed);
}

#[when(expr = "同じ対象で新しい実行を始める")]
#[when(expr = "新しい実行を始める")]
async fn start_run(world: &mut QaWorld) {
    world.run().await;
}

#[then(expr = "探索の記憶に {string} から {string} への遷移が含まれている")]
async fn has_route(world: &mut QaWorld, from: String, to: String) {
    world.run().await;
    let route = (path_of(from).to_owned(), path_of(to).to_owned());
    let routes = routes(world);
    assert!(
        routes.contains(&route),
        "the memory has {route:?}: {routes:?}"
    );
}

#[given(expr = "この対象の探索の記憶はまだ無い")]
fn no_memory(world: &mut QaWorld) {
    world.scene.memories.clear();
}

#[then(expr = "探索の記憶は空で、探索済みの要素も無い")]
async fn empty_memory(world: &mut QaWorld) {
    world.run().await;
    let kinds = world.kinds();
    assert!(
        !kinds.contains(&"memory_reset".to_owned()),
        "nothing was reset"
    );
    assert_eq!(screen_count(world), 1, "the start page only");
    assert_eq!(routes(world), [], "no edge");
    assert_eq!(world.explored(), BTreeSet::new(), "nothing explored");
}

#[given(expr = "この対象の探索の記憶のファイルが読めない形に壊れている")]
fn broken_memory(world: &mut QaWorld) {
    let origin = world.scene.origin();
    world.scene.memories.insert(origin, "{ not json".to_owned());
}

#[then(expr = "探索の記憶は空で始まっている")]
async fn starts_empty(world: &mut QaWorld) {
    world.run().await;
    assert_eq!(screen_count(world), 1, "the start page only");
    assert_eq!(routes(world), [], "no edge");
}

#[then(expr = "記憶を読めなかったことが実行記録に残っている")]
async fn reset_recorded(world: &mut QaWorld) {
    world.run().await;
    let lines = world.lines();
    let resets = lines
        .iter()
        .filter(|line| text_at(line, &["kind"]) == "memory_reset");
    let causes: Vec<String> = resets.map(|line| text_at(line, &["cause"])).collect();
    assert_eq!(causes, ["unreadable"], "the memory was unreadable");
    let aside = &world.ran().store.aside;
    let origins: Vec<&String> = aside.iter().map(|(origin, _)| origin).collect();
    assert_eq!(origins, [&world.scene.origin()], "the file was set aside");
}
