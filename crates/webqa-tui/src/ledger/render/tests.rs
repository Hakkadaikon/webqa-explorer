//! Tests of drawing the ledger screen.

use ratatui::{Terminal, backend::TestBackend, buffer::Cell};
use webqa_core::interview::issue::Kind;
use webqa_core::interview::ledger::instruction;

use super::*;
use crate::ledger::tests::sample::sample;
use crate::ledger::{Input, Notice};

fn drawn(screen: &Screen) -> String {
    let mut terminal = Terminal::new(TestBackend::new(80, 12)).unwrap();
    terminal.draw(|frame| render(screen, frame)).unwrap();
    terminal.backend().to_string()
}

/// The text of `screen` drawn in `width` columns, without the border
/// and the blanks, so that a line cut into rows reads as it was.
fn read_in(screen: &Screen, width: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, 24)).unwrap();
    terminal.draw(|frame| render(screen, frame)).unwrap();
    let cells = terminal.backend().buffer().content();
    let text: String = cells.iter().map(Cell::symbol).collect();
    let kept = |c: &char| !c.is_whitespace() && !"│┌┐└┘─".contains(*c);
    text.chars().filter(kept).collect()
}

#[test]
fn an_open_issue_reads_in_full_in_a_narrow_screen() {
    let screen = Screen::new(sample().unwrap()).fold(Input::Open);
    let entry = screen.current().unwrap();
    let said: String = instruction(&entry.issue)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    assert!(read_in(&screen, 30).contains(&said), "the instruction");
}

#[test]
fn draw_writes_the_list_to_the_stream() {
    let mut out = Vec::new();
    draw(&mut out, &Screen::new(sample().unwrap()), None).unwrap();
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("I-00000001"), "{text}");
}

/// The text of the last frame `draw` wrote, without the escapes.
fn last_frame(out: &[u8]) -> String {
    let text = String::from_utf8_lossy(out).into_owned();
    let last = text.rsplit("\x1b[2J").next().unwrap().to_owned();
    let mut escaped = false;
    let step = |c: char| {
        let inside = escaped || c == '\x1b';
        escaped = inside && !c.is_ascii_alphabetic();
        (!inside).then_some(c)
    };
    last.chars().filter_map(step).collect()
}

/// The rows of `screen` drawn `width` columns wide, as text without blanks.
fn rows_in(screen: &Screen, width: u16, height: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| render(screen, frame)).unwrap();
    let cells = terminal.backend().buffer().content();
    let row = |cells: &[Cell]| {
        let text: String = cells.iter().map(Cell::symbol).collect();
        text.replace(' ', "")
    };
    cells.chunks(usize::from(width)).map(row).collect()
}

#[test]
fn draw_uses_the_size_it_is_given() {
    let screen = Screen::new(sample().unwrap()).fold(Input::Open);
    let top = |width: u16| {
        let mut out = Vec::new();
        draw(&mut out, &screen, Some((width, 24))).unwrap();
        let frame = last_frame(&out);
        let line: String = frame.chars().take_while(|c| *c != '┐').collect();
        line.chars().count() + 1
    };
    assert_eq!((top(30), top(120)), (30, 120), "the width of the top line");
}

#[test]
fn an_open_issue_wraps_its_instruction_over_rows() {
    let screen = Screen::new(sample().unwrap()).fold(Input::Open);
    let rows = rows_in(&screen, 30, 24);
    let said = rows.iter().skip(1);
    let wrapped = said.take_while(|row| !row.contains("種類")).count();
    assert_eq!(wrapped, 6, "{rows:#?}");
}

#[test]
fn a_list_row_is_cut_at_the_width() {
    let rows = rows_in(&Screen::new(sample().unwrap()), 30, 8);
    let row = |at: usize| rows.get(at).unwrap().clone();
    assert_eq!(
        (
            row(1).starts_with("│>[]I-00000001"),
            row(2).starts_with("│[]I-00000002")
        ),
        (true, true),
        "{rows:#?}"
    );
}

/// A terminal's size is a pair of positive numbers, or not known: asking
/// for it neither fails nor gives a side of zero.
#[test]
fn the_terminal_size_is_known_or_not() {
    let size = terminal_size();
    assert!(size.is_none_or(|(width, height)| width > 0 && height > 0));
}

#[test]
fn list_marks_selected_and_cursor_rows() {
    let mut ledger = sample().unwrap();
    ledger.entries[1].selected = true;
    let text = drawn(&Screen::new(ledger));
    assert!(text.contains("│> [ ] I-00000001 判断に迷う"), "{text}");
    assert!(text.contains("│  [x] I-00000002 判断に迷う"), "{text}");
    assert!(text.contains("│  [ ] I-00000003 視認性が悪い"), "{text}");
}

#[test]
fn empty_list_says_none_and_shows_notice() {
    let screen = Screen::new(sample().unwrap())
        .fold(Input::Filter(Some(Kind::Mismatch)))
        .fold(Input::Open)
        .fold(Input::ChangedOutside);
    let text = drawn(&screen);
    assert!(text.contains("台帳 (目的のずれ)"), "{text}");
    assert!(text.contains("│なし"), "{text}");
    assert!(text.contains(notice_text(Notice::ChangedOutside)), "{text}");
}
