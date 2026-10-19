//! Tests of the ledger screen.

use webqa_core::config::raw::Op;
use webqa_core::interview::clue::Clue;
use webqa_core::interview::issue::{Act, Detail, Halt, Kind, Number};
use webqa_core::interview::ledger::Ledger;

use super::detail::lines;
use super::*;

#[path = "../../tests/sample/mod.rs"]
pub(super) mod sample;
use sample::{entry, issue, number, sample, target};

fn numbers(screen: &Screen) -> Vec<Number> {
    screen.shown().iter().map(|e| e.issue.number).collect()
}

fn screen_at(cursor: usize) -> Screen {
    (0..cursor).fold(Screen::new(sample().unwrap()), |screen, _| {
        screen.fold(Input::Down)
    })
}

#[test]
fn toggle_selects_issue() {
    let screen = screen_at(1);
    assert_eq!(screen.toggle(), Some((number(2).unwrap(), true)));
    let mut saved: Ledger = sample().unwrap();
    saved.entries[1].selected = true;
    let screen = screen.fold(Input::Saved(saved));
    assert_eq!(screen.current().map(|e| e.selected), Some(true));
    assert_eq!(screen.cursor, 1);
    assert_eq!(screen.toggle(), Some((number(2).unwrap(), false)));
}

#[test]
fn shows_external_edit_notice() {
    let screen = Screen::new(sample().unwrap()).fold(Input::ChangedOutside);
    assert_eq!(screen.notice, Some(Notice::ChangedOutside));
    assert_eq!(screen.ledger, sample().unwrap());
    assert_eq!(
        notice_text(Notice::ChangedOutside),
        "台帳が外で書き換えられたので、書き戻さなかった。開き直してから選び直す"
    );
    let screen = screen.fold(Input::Saved(sample().unwrap()));
    assert_eq!(screen.notice, None);
}

#[test]
fn filter_by_kind() {
    let screen = Screen::new(sample().unwrap()).fold(Input::Down);
    let screen = screen.fold(Input::Filter(Some(Kind::Visibility)));
    assert_eq!(numbers(&screen), vec![number(3).unwrap()]);
    assert_eq!(screen.cursor, 0);
    let screen = screen.fold(Input::Filter(None));
    assert_eq!(
        numbers(&screen),
        vec![number(1).unwrap(), number(2).unwrap(), number(3).unwrap()]
    );
    let screen = screen.fold(Input::Filter(Some(Kind::Mismatch)));
    assert_eq!(numbers(&screen), Vec::<Number>::new());
    assert_eq!(screen.toggle(), None);
    assert_eq!(screen.current(), None);
    assert_eq!(screen.fold(Input::Down).cursor, 0);
}

#[test]
fn filter_closes_issue_and_keeps_notice() {
    let screen = screen_at(1)
        .fold(Input::Open)
        .fold(Input::ChangedOutside)
        .fold(Input::Filter(None));
    assert!(!screen.open);
    assert_eq!(screen.cursor, 0);
    assert_eq!(screen.notice, Some(Notice::ChangedOutside));
}

#[test]
fn saved_pulls_cursor_into_list() {
    let screen = screen_at(2);
    let mut shorter = sample().unwrap();
    shorter.entries.pop();
    let screen = screen.fold(Input::Saved(shorter));
    assert_eq!(screen.cursor, 1);
    let empty = Ledger {
        entries: Vec::new(),
        ..sample().unwrap()
    };
    assert_eq!(screen.fold(Input::Saved(empty)).cursor, 0);
}

#[test]
fn cursor_stays_in_list() {
    assert_eq!(Screen::new(sample().unwrap()).fold(Input::Up).cursor, 0);
    assert_eq!(screen_at(2).fold(Input::Down).cursor, 2);
    assert_eq!(screen_at(2).fold(Input::Up).cursor, 1);
    let opened = Screen::new(sample().unwrap()).fold(Input::Open);
    assert!(opened.open);
    assert!(!opened.fold(Input::Close).open);
}

#[test]
fn detail_lines_show_issue() {
    let screen = screen_at(2);
    let third = screen.current().unwrap();
    assert_eq!(
        lines(third),
        vec![
            "指示 https://shop.example.test/cart の画面で、button \"確定\" について、このペルソナに見えやすく、目立つようにする(理由: 文字が小さい)",
            "種類 視認性が悪い",
            "操作 -",
            "性質 ITの慣れ: スマホは慣れている",
            "理由 文字が小さい",
            "期待 確定できる",
        ]
    );
    let first = Screen::new(sample().unwrap());
    let first = lines(first.current().unwrap());
    assert_eq!(first[2], "操作 click link \"注文\"");
    assert_eq!(first[3], "性質 -");
    assert_eq!(first[4], "理由 迷いの度合いが閾値以上、引き返した");
}

/// The lines of an issue `n` with `detail`, by way of an entry.
fn lines_of(n: u8, detail: Detail, path: Vec<Act>) -> Vec<String> {
    let mut issue = issue(n, target("link", "x"), detail).unwrap();
    issue.path = path;
    lines(&entry(issue, None))
}

fn unreached(expectation: Option<&str>) -> Detail {
    Detail::Unreached {
        halt: Halt::NoProgress,
        expectation: expectation.map(str::to_owned),
    }
}

#[test]
fn detail_lines_show_unreached_expectation() {
    let lines = |expectation| lines_of(4, unreached(expectation), Vec::new());
    assert_eq!(lines(None)[4..], ["理由 進めなくなった", "期待 -"]);
    assert_eq!(lines(Some(""))[5], "期待 -");
    assert_eq!(lines(Some("買える"))[5], "期待 買える");
}

#[test]
fn detail_lines_show_mismatch_and_empty_expectation() {
    let mismatch = Detail::Mismatch {
        purpose: "注文する".to_owned(),
        guesses: vec!["見る".to_owned(), "探す".to_owned()],
    };
    let path = vec![
        Act {
            op: Op::Click,
            target: Some(target("link", "a")),
        },
        Act {
            op: Op::Back,
            target: None,
        },
    ];
    let lines = lines_of(5, mismatch, path);
    assert_eq!(lines[2], "操作 click link \"a\" → back");
    assert_eq!(
        lines[4..],
        ["理由 書いた目的 注文する / 推測 見る、探す", "期待 -"]
    );
    let hesitant = Detail::Hesitant {
        clues: vec![Clue::PingPong],
        expectation: String::new(),
    };
    let lines = lines_of(6, hesitant, Vec::new());
    assert_eq!(lines[4..], ["理由 同じ2つの画面を行き来した", "期待 -"]);
}
