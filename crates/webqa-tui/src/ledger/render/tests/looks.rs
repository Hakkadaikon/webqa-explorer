//! Tests of how the ledger screen looks: scrolling, colors, the title and
//! the keys.

use ratatui::style::{Color, Modifier, Style};
use ratatui::{Terminal, backend::TestBackend};
use webqa_core::interview::issue::Kind;

use super::super::*;
use super::{drawn, rows_in};
use crate::ledger::tests::sample::sample;
use crate::ledger::{Input, Notice};

#[test]
fn the_list_scrolls_to_keep_the_cursor_in_view() {
    let screen = Screen::new(sample().unwrap())
        .fold(Input::Down)
        .fold(Input::Down);
    let rows = rows_in(&screen, 40, 5);
    let shown = |number: &str| rows.iter().any(|row| row.contains(number));
    assert_eq!(
        (shown("I-00000001"), shown("I-00000003")),
        (false, true),
        "{rows:#?}"
    );
}

/// The style of the cell where `text` starts in `screen` drawn 80×12.
fn style_at(screen: &Screen, text: &str) -> Style {
    let mut terminal = Terminal::new(TestBackend::new(80, 12)).unwrap();
    terminal.draw(|frame| render(screen, frame)).unwrap();
    let drawn = terminal.backend().to_string();
    let line = drawn.lines().position(|line| line.contains(text)).unwrap();
    let cells = terminal.backend().buffer().content();
    let row = cells.chunks(80).nth(line).unwrap();
    let first = text.chars().next().unwrap().to_string();
    row.iter()
        .find(|cell| cell.symbol() == first)
        .unwrap()
        .style()
}

#[test]
fn rows_take_the_color_of_their_kind_and_the_cursor_is_reversed() {
    let screen = Screen::new(sample().unwrap());
    let cursor = style_at(&screen, "> [ ] I-00000001");
    let other = style_at(&screen, "  [ ] I-00000003");
    assert_eq!(
        (cursor.fg, cursor.add_modifier.contains(Modifier::REVERSED)),
        (Some(Color::Yellow), true)
    );
    assert_eq!(
        (other.fg, other.add_modifier.contains(Modifier::REVERSED)),
        (Some(Color::Magenta), false)
    );
}

#[test]
fn every_kind_has_its_own_color() {
    let kinds = [
        Kind::Hesitant,
        Kind::Visibility,
        Kind::Unreached,
        Kind::Mismatch,
    ];
    let colors = kinds.map(kind_color);
    assert_eq!(
        colors,
        [Color::Yellow, Color::Magenta, Color::Red, Color::Blue]
    );
}

#[test]
fn the_title_counts_the_shown_and_the_selected() {
    let mut ledger = sample().unwrap();
    ledger.entries[2].selected = true;
    let screen = Screen::new(ledger).fold(Input::Filter(Some(Kind::Hesitant)));
    assert!(drawn(&screen).contains("台帳 (判断に迷う) 2件 選択1件"));
}

#[test]
fn the_keys_follow_what_is_shown() {
    let list = Screen::new(sample().unwrap());
    let open = list.clone().fold(Input::Open);
    let says = |screen: &Screen, what: &str| drawn(screen).contains(what);
    assert_eq!(
        (says(&list, "→ l Enter"), says(&list, "← h Esc")),
        (true, false)
    );
    assert_eq!(
        (says(&open, "→ l Enter"), says(&open, "← h Esc")),
        (false, true)
    );
}

#[test]
fn the_notice_is_red_under_the_list() {
    let screen = Screen::new(sample().unwrap()).fold(Input::WriteFailed);
    let style = style_at(&screen, notice_text(Notice::WriteFailed));
    assert_eq!(style.fg, Some(Color::Red));
}
