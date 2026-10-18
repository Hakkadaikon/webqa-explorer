//! The memory's side of the goal of exploration: whether anything
//! untouched is left to reach from the start.

use std::collections::BTreeSet;

use super::{NONE, below, button, buttons, click, forbid, id, page, shown};
use crate::explore::graph::{Memory, ScreenKey};
use crate::redact::Redacted;
use crate::screen::Screen;

/// "トップ" `/` with "保存" and "ヘルプ".
fn top_page() -> Redacted<Screen> {
    page("/", vec![shown("button", "保存"), shown("link", "ヘルプ")])
}

/// "トップ" and `settings` seen with `forbidden`; "保存" leads to it and
/// "ヘルプ" back to "トップ".
fn two_screens(
    settings: &Redacted<Screen>,
    forbidden: &BTreeSet<usize>,
) -> (Memory, ScreenKey, ScreenKey) {
    let mut memory = Memory::default();
    let top = memory.visit(&top_page(), &NONE);
    let to = memory.visit(settings, forbidden);
    click(&mut memory, top, &button("保存"), to);
    click(&mut memory, top, &id("link", "ヘルプ", 0), top);
    (memory, top, to)
}

/// Whether anything is left from `top`, and whether "保存" and "ヘルプ" are finished.
fn explored(memory: &Memory, top: ScreenKey) -> (bool, bool, bool) {
    (
        memory.untouched_reachable(top),
        memory.finished(top, &button("保存")),
        memory.finished(top, &id("link", "ヘルプ", 0)),
    )
}

#[test]
fn all_explored_except_forbidden_is_finished() {
    let (mut memory, top, settings) = two_screens(&buttons("/settings", &["通知"]), &NONE);
    click(&mut memory, settings, &button("通知"), settings);
    assert_eq!(
        explored(&memory, top),
        (false, true, true),
        "nothing is left and both are finished"
    );
}

#[test]
fn only_forbidden_left_is_finished() {
    let settings = buttons("/settings", &["通知", "退会"]);
    let (mut memory, top, at) = two_screens(&settings, &forbid(&[1]));
    click(&mut memory, at, &button("通知"), at);
    assert_eq!(
        (
            memory.untouched_reachable(top),
            memory.is_operated(at, &button("退会"))
        ),
        (false, false),
        "only the forbidden 退会 is left, untouched"
    );
}

#[test]
fn untouched_on_other_screen_blocks_finished() {
    let (memory, top, _) = two_screens(&buttons("/settings", &["通知"]), &NONE);
    assert_eq!(
        explored(&memory, top),
        (true, false, true),
        "通知 is left behind 保存, not behind ヘルプ"
    );
    let mut memory = Memory::default();
    let elements = vec![shown("button", "保存"), below("link", "通知設定")];
    let top = memory.visit(&page("/", elements), &NONE);
    click(&mut memory, top, &button("保存"), top);
    assert!(
        memory.untouched_reachable(top),
        "通知設定 below the viewport is left"
    );
}
