//! The memory's side of the goal of exploration: whether anything
//! untouched is left to reach from the start.

use std::collections::BTreeSet;

use super::{NONE, below, button, buttons, click, forbid, id, page, shown};
use crate::explore::graph::{Memory, ScreenKey};
use crate::redact::Redacted;
use crate::screen::Screen;

/// A screen and the positions of its forbidden elements.
type Seen<'a> = (&'a Redacted<Screen>, &'a BTreeSet<usize>);

/// "トップ" `/` with "保存" and "ヘルプ", then the buttons `more`.
fn top_page(more: &[&str]) -> Redacted<Screen> {
    let mut elements = vec![shown("button", "保存"), shown("link", "ヘルプ")];
    elements.extend(more.iter().map(|name| shown("button", name)));
    page("/", elements)
}

/// "トップ" and "設定" seen as `top` and `settings`; "保存" leads to "設定"
/// and "ヘルプ" back to "トップ".
fn seen_both(top: Seen<'_>, settings: Seen<'_>) -> (Memory, ScreenKey, ScreenKey) {
    let mut memory = Memory::default();
    let from = memory.visit(top.0, top.1);
    let to = memory.visit(settings.0, settings.1);
    click(&mut memory, from, &button("保存"), to);
    click(&mut memory, from, &id("link", "ヘルプ", 0), from);
    (memory, from, to)
}

/// [`seen_both`] with "トップ" of "保存" and "ヘルプ" only.
fn two_screens(
    settings: &Redacted<Screen>,
    forbidden: &BTreeSet<usize>,
) -> (Memory, ScreenKey, ScreenKey) {
    seen_both((&top_page(&[]), &NONE), (settings, forbidden))
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
    let top_page = top_page(&["退会"]);
    let settings = buttons("/settings", &["通知"]);
    let (mut memory, top, at) = seen_both((&top_page, &forbid(&[2])), (&settings, &NONE));
    click(&mut memory, at, &button("通知"), at);
    assert_eq!(
        explored(&memory, top),
        (false, true, true),
        "nothing but the forbidden 退会 is left, and both are finished"
    );
    assert!(
        !memory.is_operated(top, &button("退会")),
        "退会 stays untouched"
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
