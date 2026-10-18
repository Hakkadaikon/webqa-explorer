//! An empty memory, recording, and `Debug`.

use super::{NONE, VIEW, button, buttons, click};
use crate::explore::graph::{ElementOp, Memory, ScreenKey};
use crate::explore::select::{Candidates, candidates};

/// A key of another memory, at position `n`.
fn foreign(n: usize) -> ScreenKey {
    let mut other = Memory::default();
    let paths = ["/a", "/b", "/c", "/d"];
    let mut key = other.visit(&buttons("/", &[]), &NONE);
    for path in paths.iter().take(n) {
        key = other.visit(&buttons(path, &[]), &NONE);
    }
    key
}

#[test]
fn empty_memory_knows_nothing() {
    let save = button("保存");
    let key = foreign(0);
    let mut memory = Memory::default();
    assert_eq!(memory.screen_count(), 0, "no screens");
    assert_eq!(memory.edge_count(), 0, "no edges");
    assert!(!memory.is_operated(key, &save), "unknown screen");
    assert!(!memory.finished(key, &save), "unknown screen");
    assert!(!memory.untouched_reachable(key), "unknown screen");
    assert!(!click(&mut memory, key, &save, key), "nothing to record");
    assert_eq!(memory.edge_count(), 0, "no edge was added");
    let saved = buttons("/", &["保存"]);
    assert_eq!(
        candidates(&memory, key, &saved, VIEW),
        Candidates::Moves,
        "no forbidden marks to read"
    );
}

/// "トップ" with 保存 and "設定" with 通知.
fn two_screens() -> (Memory, ScreenKey, ScreenKey) {
    let mut memory = Memory::default();
    let top = memory.visit(&buttons("/", &["保存"]), &NONE);
    let settings = memory.visit(&buttons("/settings", &["通知"]), &NONE);
    (memory, top, settings)
}

fn same_edge_once() {
    let save = button("保存");
    let (mut memory, top, settings) = two_screens();
    assert!(click(&mut memory, top, &save, settings), "first time");
    assert!(!click(&mut memory, top, &save, settings), "second time");
    assert_eq!(memory.edge_count(), 1, "the same edge once");
    assert!(!click(&mut memory, top, &save, top), "another target");
    assert_eq!(memory.edge_count(), 2, "another edge");
    let input = memory.record(top, &save, ElementOp::Input, top);
    assert!(!input, "another operation");
    assert_eq!(memory.edge_count(), 3, "another edge");
}

fn unknown_parts_change_nothing() {
    let (mut memory, top, settings) = two_screens();
    let notice = button("通知");
    assert!(!click(&mut memory, top, &notice, settings), "not on トップ");
    assert!(!memory.is_operated(top, &notice), "not on トップ");
    assert!(
        !click(&mut memory, top, &button("保存"), foreign(3)),
        "unknown target"
    );
    assert!(
        !memory.is_operated(top, &button("保存")),
        "nothing recorded"
    );
    let source = click(&mut memory, foreign(3), &button("保存"), top);
    assert!(!source, "unknown source");
    assert_eq!(memory.edge_count(), 0, "no edges");
}

#[test]
fn record_reports_new_operation_once() {
    same_edge_once();
    unknown_parts_change_nothing();
}

#[test]
fn memory_debug_hides_names() {
    let mut memory = Memory::default();
    let top = memory.visit(&buttons("/", &["保存"]), &NONE);
    click(&mut memory, top, &button("保存"), top);
    let debug = format!("{memory:?}");
    assert!(!debug.contains("保存"), "names are hidden: {debug}");
}
