//! Draws the ledger screen.

use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::terminal::{Clear, ClearType, size};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{List, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};
use webqa_core::interview::issue::Kind;

use crate::follow::area;
use crate::style::{framed, hints, labelled};

use super::detail::{lines, row};
use super::{Screen, notice_text};
use webqa_core::interview::ledger::Entry;

/// Draws `screen` over the whole frame: the list or the open issue, then
/// the notice, if any, and the keys.
pub fn render(screen: &Screen, frame: &mut Frame) {
    let [body, notice, keys] = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(u16::from(screen.notice.is_some())),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let open = screen.current().filter(|_| screen.open);
    draw_body(screen, open, frame, body);
    let told = screen.notice.map(notice_text).unwrap_or_default();
    frame.render_widget(Paragraph::new(told.red().bold()), notice);
    let hint: &[_] = if open.is_some() {
        &OPEN_KEYS
    } else {
        &LIST_KEYS
    };
    frame.render_widget(Paragraph::new(hints(hint)), keys);
}

/// Draws the open issue, wrapped so that it reads in full, or the list,
/// scrolled to keep the cursor in view, its rows cut at the width.
fn draw_body(screen: &Screen, open: Option<&Entry>, frame: &mut Frame, body: Rect) {
    if let Some(entry) = open {
        let text: Vec<Line> = lines(entry).iter().map(|line| labelled(line)).collect();
        let view = Paragraph::new(text).block(framed(&entry.issue.number.to_string()));
        frame.render_widget(view.wrap(Wrap { trim: false }), body);
    } else {
        let mut state = ListState::default().with_selected(Some(screen.cursor));
        frame.render_stateful_widget(list(screen), body, &mut state);
    }
}

/// The keys of the list, and of an open issue.
const LIST_KEYS: [(&str, &str); 6] = [
    ("↑↓ jk", "移動"),
    ("→ l Enter", "開く"),
    ("Space", "選ぶ"),
    ("1-4", "種類"),
    ("0", "全部"),
    ("q", "終わる"),
];
const OPEN_KEYS: [(&str, &str); 4] = [
    ("↑↓ jk", "前後の課題"),
    ("← h Esc", "一覧へ"),
    ("Space", "選ぶ"),
    ("q", "終わる"),
];

/// The color of each kind of issue.
const fn kind_color(kind: Kind) -> Color {
    match kind {
        Kind::Hesitant => Color::Yellow,
        Kind::Visibility => Color::Magenta,
        Kind::Unreached => Color::Red,
        Kind::Mismatch => Color::Blue,
    }
}

/// The size of the terminal, if it has one. A constant, not a function, as a
/// real terminal's size cannot be fixed by a test.
pub const TERMINAL_SIZE: fn() -> Option<(u16, u16)> = || size().ok();

/// Clears the screen and draws `screen` on it, `size` wide and high
/// (columns, rows); 80×24 when it is not known.
///
/// # Errors
///
/// When `out` cannot be written.
pub fn draw<W: Write>(out: &mut W, screen: &Screen, size: Option<(u16, u16)>) -> io::Result<()> {
    let mut frame = Vec::new();
    drop(queue!(frame, Clear(ClearType::All), MoveTo(0, 0)));
    let viewport = Viewport::Fixed(area(size));
    let backend = CrosstermBackend::new(&mut frame);
    let options = TerminalOptions { viewport };
    let drawn = Terminal::with_options(backend, options)
        .and_then(|mut terminal| terminal.draw(|frame| render(screen, frame)).map(drop));
    drop(drawn);
    out.write_all(&frame).and_then(|()| out.flush())
}

fn list_title(screen: &Screen) -> String {
    let kind = screen.filter.map(|kind| format!(" ({})", kind.label()));
    let selected = screen.ledger.entries.iter().filter(|entry| entry.selected);
    format!(
        "台帳{} {}件 選択{}件(全体)",
        kind.unwrap_or_default(),
        screen.shown().len(),
        selected.count()
    )
}

fn list(screen: &Screen) -> List<'static> {
    let styled = |(at, entry): (usize, &Entry)| {
        let color = kind_color(entry.issue.kind());
        Line::styled(row(entry, at == screen.cursor), Style::new().fg(color))
    };
    let mut rows: Vec<Line> = screen.shown().into_iter().enumerate().map(styled).collect();
    if rows.is_empty() {
        rows.push(Line::from("なし"));
    }
    List::new(rows)
        .block(framed(&list_title(screen)))
        .highlight_style(Style::new().reversed().bold())
}

#[cfg(test)]
mod tests;
