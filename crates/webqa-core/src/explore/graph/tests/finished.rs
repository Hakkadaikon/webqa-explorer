//! Whether an element is explored follows the screens it leads to.

use std::collections::BTreeSet;

use super::{NONE, button, buttons, click, forbid, id, page, shown};
use crate::explore::graph::{Memory, ScreenKey};
use crate::redact::Redacted;
use crate::screen::Screen;

/// "トップ" → "設定" → "メール" by clicking "設定" and "通知".
fn chain(mail: &Redacted<Screen>, forbidden: &BTreeSet<usize>) -> (Memory, ScreenKey, ScreenKey) {
    let mut memory = Memory::default();
    let top = memory.visit(&buttons("/", &["設定"]), &NONE);
    let settings = memory.visit(&buttons("/settings", &["通知"]), &NONE);
    let m = memory.visit(mail, forbidden);
    click(&mut memory, top, &button("設定"), settings);
    click(&mut memory, settings, &button("通知"), m);
    (memory, top, m)
}

fn one_step_ahead() {
    let top_page = buttons("/", &["設定"]);
    let open = button("設定");
    let mut memory = Memory::default();
    let top = memory.visit(&top_page, &NONE);
    let settings = memory.visit(&buttons("/settings", &["通知"]), &NONE);
    click(&mut memory, top, &open, settings);
    assert_eq!(memory.visit(&top_page, &NONE), top, "the same screen again");
    assert!(memory.is_operated(top, &open), "clicked");
    assert!(!memory.finished(top, &open), "通知 is untouched");
}

fn two_steps_ahead() {
    let (memory, top, _) = chain(&buttons("/settings/mail", &["解除"]), &NONE);
    assert!(
        !memory.finished(top, &button("設定")),
        "解除 two steps ahead"
    );
}

fn cycle_stops() {
    let mail = page(
        "/settings/mail",
        vec![shown("button", "解除"), shown("link", "トップ")],
    );
    let (mut memory, top, m) = chain(&mail, &NONE);
    click(&mut memory, m, &button("解除"), m);
    click(&mut memory, m, &id("link", "トップ", 0), top);
    assert!(memory.finished(top, &button("設定")), "the cycle stops");
}

fn only_forbidden_left() {
    let (memory, top, _) = chain(&buttons("/settings/mail", &["解除"]), &forbid(&[0]));
    assert!(
        memory.finished(top, &button("設定")),
        "only a forbidden element is left"
    );
}

#[test]
fn finished_follows_reachable_screens() {
    one_step_ahead();
    two_steps_ahead();
    cycle_stops();
    only_forbidden_left();
}

/// "設定" on "トップ" leads to an explored "完了" (0) and to "設定" with 通知 left (1).
///
/// The edges are recorded in `order`; returns whether "設定" is finished.
fn two_branches(order: [usize; 2]) -> bool {
    let mut memory = Memory::default();
    let top = memory.visit(&buttons("/", &["設定"]), &NONE);
    let done = memory.visit(&buttons("/done", &["完了"]), &NONE);
    let settings = memory.visit(&buttons("/settings", &["通知"]), &NONE);
    click(&mut memory, done, &button("完了"), done);
    let targets = [done, settings];
    for i in order {
        click(&mut memory, top, &button("設定"), targets[i]);
    }
    memory.finished(top, &button("設定"))
}

#[test]
fn finished_needs_every_branch_explored() {
    assert!(!two_branches([0, 1]), "explored branch first");
    assert!(!two_branches([1, 0]), "unexplored branch first");
}
