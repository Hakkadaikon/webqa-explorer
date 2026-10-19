//! The memory a previous run left.

use std::collections::BTreeSet;

use webqa_core::explore::{ElementOp, Memory, ScreenKey};
use webqa_core::redact::{Redacted, Redactor};
use webqa_core::screen::{ElementId, Screen, element_ids};

use crate::page::{ElementSpec, PageSpec};
use crate::scene::{Scene, found};

/// A screen of the memory, as the first allowed origin shows it.
fn screen_of(origin: &str, (path, elements): &(&str, &[(&str, &str)])) -> Redacted<Screen> {
    let page = elements
        .iter()
        .fold(PageSpec::at(path), |page, (role, name)| {
            page.with(ElementSpec::new(role, name))
        });
    Redactor::default().screen(page.build(origin).screen)
}

/// The first element of the screen named `name`. None fails the scenario.
fn element_named(screen: &Redacted<Screen>, name: &str) -> ElementId {
    let named = element_ids(screen).into_iter().find(|id| id.name == name);
    found(named, "the screen has the element")
}

/// Adds the edge `(from, name, to)`: the click on the element of the
/// screen `from`, which leads to the screen `to`.
fn connect(
    memory: &mut Memory,
    shown: &[Redacted<Screen>],
    keys: &[ScreenKey],
    edge: (usize, &str, usize),
) {
    let (from, name, to) = edge;
    let screen = found(shown.get(from), "the edge starts at a screen");
    let ends = (found(keys.get(from), "from"), found(keys.get(to), "to"));
    memory.record(
        *ends.0,
        &element_named(screen, name),
        ElementOp::Click,
        *ends.1,
    );
}

/// Puts a memory file for the first allowed origin in `scene.memories`, as
/// a previous run would have left it: `screens` as (path, (role, name)
/// elements), and `edges` as (from screen, element name, to screen)
/// clicks, which mark the element operated. A name with no element fails
/// the scenario.
pub fn remember(
    scene: &mut Scene,
    screens: &[(&str, &[(&str, &str)])],
    edges: &[(usize, &str, usize)],
) {
    let origin = scene.origin();
    let shown: Vec<_> = screens.iter().map(|s| screen_of(&origin, s)).collect();
    let mut memory = Memory::default();
    let keys: Vec<_> = shown
        .iter()
        .map(|screen| memory.visit(screen, &BTreeSet::new()))
        .collect();
    for &edge in edges {
        connect(&mut memory, &shown, &keys, edge);
    }
    let json = memory.to_json(&origin).to_string();
    scene.memories.insert(origin, json);
}

/// The memory a scene left for `origin`, read back.
fn read_back(scene: &Scene, origin: &str) -> Memory {
    let text = scene.memories.get(origin);
    let read = text.map(|text| Memory::from_json(text, origin));
    read.and_then(Result::ok).unwrap_or_default()
}

/// Checks that the memory left reads back as a previous run left it.
pub fn check() {
    let origin = "https://app.example.test";
    let mut scene = Scene::default();
    scene.origins.push(origin.to_owned());
    let home: &[(&str, &str)] = &[("button", "保存"), ("link", "ヘルプ")];
    let settings: &[(&str, &str)] = &[("button", "通知")];
    let screens = [("/", home), ("/settings", settings)];
    remember(&mut scene, &screens, &[(0, "保存", 1)]);
    let mut memory = read_back(&scene, origin);
    let counts = (memory.screen_count(), memory.edge_count());
    assert_eq!(counts, (2, 1), "two screens, one edge");
    let first = screen_of(origin, &("/", home));
    let key = memory.visit(&first, &BTreeSet::new());
    let saved = element_named(&first, "保存");
    assert!(memory.is_operated(key, &saved), "the button is operated");
}
