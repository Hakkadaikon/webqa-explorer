//! Tests of the candidate tiers.

use super::*;
use crate::explore::graph::ElementOp;
use crate::explore::graph::tests::{NONE, VIEW, at, below, candidate, forbid, id, page, shown};

mod own_screen;

/// "トップ" with 保存 and ヘルプ, and "設定" with 通知; 保存 leads to 設定.
fn top_and_settings() -> (Memory, ScreenKey, ScreenKey, Redacted<Screen>) {
    let top_page = page("/", vec![shown("button", "保存"), shown("link", "ヘルプ")]);
    let mut memory = Memory::default();
    let top = memory.visit(&top_page, &NONE);
    let settings = memory.visit(&page("/settings", vec![shown("button", "通知")]), &NONE);
    memory.record(top, &id("button", "保存", 0), ElementOp::Click, settings);
    (memory, top, settings, top_page)
}

#[test]
fn untouched_before_unfinished_branch() {
    let (memory, top, _, top_page) = top_and_settings();
    assert!(
        !memory.finished(top, &id("button", "保存", 0)),
        "通知 on 設定 is untouched"
    );
    assert_eq!(
        candidates(&memory, top, &top_page, VIEW),
        Candidates::Untouched(vec![candidate(2, "link", "ヘルプ", 0)]),
        "only the untouched ヘルプ"
    );
}

#[test]
fn unfinished_branch_before_back_and_scroll() {
    let (mut memory, top, _, top_page) = top_and_settings();
    let help = id("link", "ヘルプ", 0);
    memory.record(top, &help, ElementOp::Click, top);
    assert!(memory.finished(top, &help), "no move from ヘルプ");
    assert_eq!(
        candidates(&memory, top, &top_page, VIEW),
        Candidates::Unfinished(vec![candidate(1, "button", "保存", 0)]),
        "保存 still leads to 通知"
    );
}

#[test]
fn back_or_scroll_when_all_finished() {
    let (mut memory, _, settings, _) = top_and_settings();
    let settings_page = page("/settings", vec![shown("button", "通知")]);
    let notice = id("button", "通知", 0);
    memory.record(settings, &notice, ElementOp::Click, settings);
    assert_eq!(
        candidates(&memory, settings, &settings_page, VIEW),
        Candidates::Moves,
        "ヘルプ on トップ is not a branch of 設定"
    );
    assert!(memory.finished(settings, &notice), "no move from 通知");

    let (mut memory, _, settings, _) = top_and_settings();
    let mail = memory.visit(
        &page("/settings/mail", vec![shown("button", "解除")]),
        &NONE,
    );
    memory.record(mail, &id("button", "解除", 0), ElementOp::Click, mail);
    memory.record(settings, &notice, ElementOp::Click, mail);
    assert_eq!(
        candidates(&memory, settings, &settings_page, VIEW),
        Candidates::Moves,
        "メール has nothing left"
    );
}

#[test]
fn payment_elements_never_candidates() {
    let cart = page("/cart", vec![shown("button", "購入する")]);
    let mut memory = Memory::default();
    let key = memory.visit(&cart, &forbid(&[0]));
    assert_eq!(
        candidates(&memory, key, &cart, VIEW),
        Candidates::Moves,
        "購入する is forbidden"
    );

    let cart = page(
        "/cart",
        vec![shown("button", "購入する"), shown("link", "ヘルプ")],
    );
    let key = memory.visit(&cart, &forbid(&[0]));
    assert_eq!(
        candidates(&memory, key, &cart, VIEW),
        Candidates::Untouched(vec![candidate(2, "link", "ヘルプ", 0)]),
        "only ヘルプ"
    );
}

#[test]
fn forbidden_untouched_excluded() {
    let account_page = page(
        "/account",
        vec![shown("button", "保存"), shown("button", "退会")],
    );
    let mut memory = Memory::default();
    let account = memory.visit(&account_page, &forbid(&[1]));
    memory.visit(&page("/", vec![shown("link", "ヘルプ")]), &NONE);
    memory.record(account, &id("button", "保存", 0), ElementOp::Click, account);
    assert_eq!(
        candidates(&memory, account, &account_page, VIEW),
        Candidates::Moves,
        "退会 is forbidden"
    );
    assert!(
        !memory.untouched_reachable(account),
        "only the forbidden 退会 is untouched"
    );
    assert!(
        !memory.is_operated(account, &id("button", "退会", 0)),
        "退会 stays untouched"
    );
}

#[test]
fn empty_screen_offers_moves() {
    let empty = page("/empty", Vec::new());
    let mut memory = Memory::default();
    let key = memory.visit(&empty, &NONE);
    assert_eq!(memory.screen_count(), 1, "an empty screen is a screen");
    assert_eq!(
        candidates(&memory, key, &empty, VIEW),
        Candidates::Moves,
        "nothing to pick"
    );
    assert!(!memory.untouched_reachable(key), "nothing untouched");

    let far = page("/far", vec![below("button", "保存")]);
    let key = memory.visit(&far, &NONE);
    assert_eq!(
        candidates(&memory, key, &far, VIEW),
        Candidates::Moves,
        "scroll to show 保存"
    );
    assert!(memory.untouched_reachable(key), "保存 is untouched");
}

#[test]
fn candidates_follow_viewport_numbers() {
    let full = page(
        "/",
        vec![
            at("button", "上", -50),
            shown("button", "保存"),
            below("link", "下"),
            shown("link", "ヘルプ"),
        ],
    );
    let mut memory = Memory::default();
    let key = memory.visit(&full, &NONE);
    assert_eq!(
        candidates(&memory, key, &full, VIEW),
        Candidates::Untouched(vec![
            candidate(1, "button", "保存", 0),
            candidate(2, "link", "ヘルプ", 0),
        ]),
        "numbers count only the viewport"
    );
    let unknown = page("/", vec![shown("button", "削除")]);
    assert_eq!(
        candidates(&memory, key, &unknown, VIEW),
        Candidates::Moves,
        "削除 is not in the memory"
    );
    let mixed = page("/", vec![shown("button", "削除"), shown("button", "保存")]);
    assert_eq!(
        candidates(&memory, key, &mixed, VIEW),
        Candidates::Untouched(vec![candidate(2, "button", "保存", 0)]),
        "削除 is left out"
    );
}

#[test]
fn forbidden_after_operation_leaves_second_tier() {
    let (mut memory, top, _, top_page) = top_and_settings();
    memory.record(top, &id("link", "ヘルプ", 0), ElementOp::Click, top);
    assert_eq!(
        memory.visit(&top_page, &forbid(&[0])),
        top,
        "the same screen"
    );
    assert_eq!(
        candidates(&memory, top, &top_page, VIEW),
        Candidates::Moves,
        "保存 is now forbidden"
    );
}
