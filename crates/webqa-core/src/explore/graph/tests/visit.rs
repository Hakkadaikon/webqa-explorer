//! Visiting finds the same screen and updates its elements.

use super::{NONE, VIEW, below, button, buttons, candidate, click, forbid, id, page, shown};
use crate::explore::graph::{Memory, ScreenKey};
use crate::explore::select::{Candidates, candidates};
use crate::screen::Element;

fn named(element: Element, value: &str) -> Element {
    Element {
        value: Some(value.to_owned()),
        ..element
    }
}

/// The query, the values and the boxes do not make a new screen.
fn query_value_and_box_ignored() {
    let a = page(
        "/?q=1",
        vec![
            named(shown("textbox", "名前"), "a"),
            shown("button", "保存"),
        ],
    );
    let b = page(
        "/?q=2",
        vec![
            named(below("textbox", "名前"), "b"),
            below("button", "保存"),
        ],
    );
    let mut memory = Memory::default();
    let key = memory.visit(&a, &NONE);
    assert_eq!(memory.visit(&b, &NONE), key, "the same screen");
    assert_eq!(memory.screen_count(), 1, "one screen");
    let added = page(
        "/",
        vec![
            shown("textbox", "名前"),
            shown("button", "保存"),
            shown("link", "ヘルプ"),
        ],
    );
    let next = memory.visit(&added, &NONE);
    assert_eq!(next, ScreenKey(1), "another element set");
    assert_eq!(memory.screen_count(), 2, "two screens");
}

/// A second element with the same name keeps the screen and adds an entry.
fn same_name_adds_an_element() {
    let one = buttons("/", &["保存"]);
    let two = buttons("/", &["保存", "保存"]);
    let mut memory = Memory::default();
    let top = memory.visit(&one, &NONE);
    click(&mut memory, top, &button("保存"), top);
    assert_eq!(memory.visit(&two, &NONE), top, "the same (role, name) set");
    let second = vec![candidate(2, "button", "保存", 1)];
    let tier = candidates(&memory, top, &two, VIEW);
    assert_eq!(
        tier,
        Candidates::Untouched(second),
        "the second 保存 is new"
    );
    click(&mut memory, top, &id("button", "保存", 1), top);
    assert_eq!(memory.visit(&one, &NONE), top, "back to one 保存");
    let kept = memory.is_operated(top, &id("button", "保存", 1));
    assert!(kept, "missing elements keep their marks");
    assert_eq!(memory.screen_count(), 1, "still one screen");
}

/// Forbidden marks follow the latest visit; out-of-range numbers are ignored.
fn forbidden_follows_the_latest_visit() {
    let saved = buttons("/", &["保存"]);
    let tier = Candidates::Untouched(vec![candidate(1, "button", "保存", 0)]);
    let mut memory = Memory::default();
    let top = memory.visit(&saved, &forbid(&[0]));
    let offered = |memory: &Memory| candidates(memory, top, &saved, VIEW);
    assert_eq!(offered(&memory), Candidates::Moves, "forbidden");
    assert_eq!(memory.visit(&saved, &NONE), top, "same screen");
    assert_eq!(offered(&memory), tier, "no longer forbidden");
    assert_eq!(memory.visit(&saved, &forbid(&[9])), top, "same screen");
    assert_eq!(offered(&memory), tier, "out-of-range numbers are ignored");
}

/// Elements missing from the latest visit keep their forbidden marks.
fn missing_elements_keep_forbidden() {
    let one = buttons("/", &["保存"]);
    let mut memory = Memory::default();
    let top = memory.visit(&one, &NONE);
    click(&mut memory, top, &button("保存"), top);
    let two = buttons("/", &["保存", "保存"]);
    assert_eq!(memory.visit(&two, &forbid(&[1])), top, "the same screen");
    assert_eq!(memory.visit(&one, &NONE), top, "back to one 保存");
    let left = memory.untouched_reachable(top);
    assert!(!left, "the second 保存 stays forbidden");
}

#[test]
fn visit_reuses_the_same_screen() {
    query_value_and_box_ignored();
    same_name_adds_an_element();
    forbidden_follows_the_latest_visit();
    missing_elements_keep_forbidden();
}
