//! Tests of a branch that returns to its own screen.

use super::super::*;
use crate::explore::graph::ElementOp;
use crate::explore::graph::tests::{NONE, VIEW, below, candidate, id, page, shown};

/// "トップ" with 設定 in view and 下 below it, and "設定" with a link back.
fn returning() -> (
    Memory,
    ScreenKey,
    ScreenKey,
    Redacted<Screen>,
    Redacted<Screen>,
) {
    let top_page = page("/", vec![shown("button", "設定"), below("button", "下")]);
    let settings_page = page("/settings", vec![shown("link", "トップ")]);
    let mut memory = Memory::default();
    let top = memory.visit(&top_page, &NONE);
    let settings = memory.visit(&settings_page, &NONE);
    memory.record(top, &id("button", "設定", 0), ElementOp::Click, settings);
    memory.record(settings, &id("link", "トップ", 0), ElementOp::Click, top);
    (memory, top, settings, top_page, settings_page)
}

#[test]
fn return_to_own_screen_falls_to_moves() {
    let (memory, top, _, top_page, _) = returning();
    assert!(
        memory.finished(top, &id("button", "設定", 0)),
        "下 on トップ is not a branch of 設定"
    );
    assert_eq!(
        candidates(&memory, top, &top_page, VIEW),
        Candidates::Moves,
        "scroll to show 下"
    );
    assert!(memory.untouched_reachable(top), "下 is untouched");
}

#[test]
fn return_leads_back_to_unseen_element() {
    let (memory, _, settings, _, settings_page) = returning();
    assert!(
        !memory.finished(settings, &id("link", "トップ", 0)),
        "back to トップ still leads to 下"
    );
    assert_eq!(
        candidates(&memory, settings, &settings_page, VIEW),
        Candidates::Unfinished(vec![candidate(1, "link", "トップ", 0)]),
        "go back to トップ to scroll"
    );
}
