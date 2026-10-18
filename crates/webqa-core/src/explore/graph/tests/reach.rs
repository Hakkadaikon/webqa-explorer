//! Untouched elements reachable along edges.

use super::{NONE, button, buttons, click, id, page, shown};
use crate::explore::graph::{Memory, ScreenKey};

/// "トップ" and "設定" linked both ways, with every element operated.
fn cycle() -> (Memory, ScreenKey, ScreenKey) {
    let mut memory = Memory::default();
    let top = memory.visit(&buttons("/", &["設定"]), &NONE);
    let settings = memory.visit(&page("/settings", vec![shown("link", "トップ")]), &NONE);
    click(&mut memory, top, &button("設定"), settings);
    click(&mut memory, settings, &id("link", "トップ", 0), top);
    (memory, top, settings)
}

#[test]
fn untouched_reachable_follows_edges_and_stops_on_cycles() {
    let (mut memory, top, settings) = cycle();
    assert!(!memory.untouched_reachable(top), "all operated from トップ");
    let from_settings = memory.untouched_reachable(settings);
    assert!(!from_settings, "all operated from 設定");

    let alone = memory.visit(&buttons("/alone", &["孤立"]), &NONE);
    assert!(memory.untouched_reachable(alone), "孤立 is untouched");
    assert!(!memory.untouched_reachable(top), "no edge to /alone");

    let links = vec![shown("link", "トップ"), shown("link", "トップ")];
    let again = memory.visit(&page("/settings", links), &NONE);
    assert_eq!(again, settings, "the same screen");
    assert!(memory.untouched_reachable(top), "the second トップ link");
}
