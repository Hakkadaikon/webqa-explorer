//! Draws the ledger screen.

use ratatui::{
    Frame,
    widgets::{Block, Paragraph},
};

use super::detail::{lines, row};
use super::{Screen, notice_text};

/// Draws `screen` over the whole frame.
pub fn render(screen: &Screen, frame: &mut Frame) {
    let (title, text) = screen.current().filter(|_| screen.open).map_or_else(
        || (list_title(screen), list(screen)),
        |entry| (entry.issue.number.to_string(), lines(entry)),
    );
    let view = Paragraph::new(text.join("\n")).block(Block::bordered().title(title));
    frame.render_widget(view, frame.area());
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
    use ratatui::{Terminal, backend::TestBackend};
    use webqa_core::interview::issue::Kind;

    use super::*;
    use crate::ledger::tests::sample::sample;
    use crate::ledger::{Input, Notice};

    fn drawn(screen: &Screen) -> String {
        let mut terminal = Terminal::new(TestBackend::new(80, 6)).unwrap();
        terminal.draw(|frame| render(screen, frame)).unwrap();
        terminal.backend().to_string()
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
