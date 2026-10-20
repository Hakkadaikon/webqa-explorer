//! Draws the ledger screen.

use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::terminal::{Clear, ClearType, size};
use ratatui::backend::CrosstermBackend;
use ratatui::{
    Frame, Terminal, TerminalOptions, Viewport,
    widgets::{Block, Paragraph, Wrap},
};

use crate::follow::area;

use super::detail::{lines, row};
use super::{Screen, notice_text};

/// Draws `screen` over the whole frame. An open issue wraps its lines so
/// that it reads in full; the list cannot scroll, so its rows are cut at
/// the width.
pub fn render(screen: &Screen, frame: &mut Frame) {
    let open = screen.current().filter(|_| screen.open);
    let (title, text) = open.map_or_else(
        || (list_title(screen), list(screen)),
        |entry| (entry.issue.number.to_string(), lines(entry)),
    );
    let view = Paragraph::new(text.join("\n")).block(Block::bordered().title(title));
    let view = if open.is_some() {
        view.wrap(Wrap { trim: false })
    } else {
        view
    };
    frame.render_widget(view, frame.area());
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
    format!("台帳{}", kind.unwrap_or_default())
}

fn list(screen: &Screen) -> Vec<String> {
    let mut rows: Vec<String> = screen
        .shown()
        .into_iter()
        .enumerate()
        .map(|(at, entry)| row(entry, at == screen.cursor))
        .collect();
    if rows.is_empty() {
        rows.push("なし".to_owned());
    }
    rows.extend(screen.notice.map(|notice| notice_text(notice).to_owned()));
    rows
}

#[cfg(test)]
mod tests;
