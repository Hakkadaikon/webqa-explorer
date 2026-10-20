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

/// Draws `screen` over the whole frame.
pub fn render(screen: &Screen, frame: &mut Frame) {
    let (title, text) = screen.current().filter(|_| screen.open).map_or_else(
        || (list_title(screen), list(screen)),
        |entry| (entry.issue.number.to_string(), lines(entry)),
    );
    let view = Paragraph::new(text.join("\n"))
        .wrap(Wrap { trim: false })
        .block(Block::bordered().title(title));
    frame.render_widget(view, frame.area());
}

/// The size of the terminal, if it has one.
#[must_use]
pub fn terminal_size() -> Option<(u16, u16)> {
    size().ok()
}

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
mod tests {
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

    #[test]
    fn draw_uses_the_size_it_is_given() {
        let screen = Screen::new(sample().unwrap()).fold(Input::Open);
        let mut out = Vec::new();
        draw(&mut out, &screen, Some((30, 24))).unwrap();
        let text = String::from_utf8_lossy(&out);
        let plain: String = text.split("\x1b[2J").last().unwrap().to_owned();
        let mut wide = Vec::new();
        draw(&mut wide, &screen, Some((120, 24))).unwrap();
        assert_ne!(plain, String::from_utf8_lossy(&wide), "{plain}");
    }

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
}
