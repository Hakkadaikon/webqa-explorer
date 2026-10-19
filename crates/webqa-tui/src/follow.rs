//! What the viewer has read of one run, and where to draw it.

use std::io::{self, Write};
use std::mem::take;

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::terminal::{Clear, ClearType, size};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use webqa_core::event::Line;

use crate::tail::Tail;
use crate::view::{State, render};

/// What the viewer has read of one run.
#[derive(Debug, Default)]
pub struct Follow {
    tail: Tail,
    state: State,
    finished: bool,
}

impl Follow {
    /// Folds the lines `chunk` completes; whether the state changed.
    pub fn read(&mut self, chunk: &[u8]) -> bool {
        let before = self.state.clone();
        let fold = |state: State, line: &Line| state.fold(line);
        self.state = self
            .tail
            .feed(chunk)
            .iter()
            .fold(take(&mut self.state), fold);
        self.finished = self.state.finished.is_some();
        self.state != before
    }

    /// The state to draw.
    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }

    /// Whether `run_finished` was read.
    #[must_use]
    pub const fn finished(&self) -> bool {
        self.finished
    }
}

/// The area to draw in: the terminal's size, or 80×24 when unknown
/// (a size with a zero side is no size).
#[must_use]
pub fn area(size: Option<(u16, u16)>) -> Rect {
    let known = size.filter(|&(width, height)| width > 0 && height > 0);
    let (width, height) = known.unwrap_or((80, 24));
    Rect::new(0, 0, width, height)
}

/// Clears the screen and draws `state` on it, as large as the terminal.
///
/// # Errors
///
/// When `out` cannot be written.
pub fn draw<W: Write>(out: &mut W, state: &State) -> io::Result<()> {
    let mut frame = Vec::new();
    paint(&mut frame, state);
    out.write_all(&frame).and_then(|()| out.flush())
}

/// Writes the screen into memory, where a write cannot fail.
fn paint(frame: &mut Vec<u8>, state: &State) {
    drop(queue!(frame, Clear(ClearType::All), MoveTo(0, 0)));
    let viewport = Viewport::Fixed(area(size().ok()));
    let backend = CrosstermBackend::new(frame);
    let options = TerminalOptions { viewport };
    let drawn = Terminal::with_options(backend, options)
        .and_then(|mut terminal| terminal.draw(|frame| render(state, frame)).map(drop));
    drop(drawn);
}

#[cfg(test)]
mod tests {
    use webqa_core::event::parse_line;

    use super::*;
    use crate::EXAMPLE;

    fn folded(lines: &[&str]) -> State {
        let fold = |state: State, text: &&str| state.fold(&parse_line(text).unwrap());
        lines.iter().fold(State::default(), fold)
    }

    fn joined(lines: &[&str]) -> Vec<u8> {
        lines
            .iter()
            .flat_map(|line| format!("{line}\n").into_bytes())
            .collect()
    }

    #[test]
    fn follow_folds_appended_lines() {
        let mut follow = Follow::default();
        assert!(follow.read(&joined(&EXAMPLE[..2])));
        assert!(follow.read(&joined(&EXAMPLE[2..])));
        assert_eq!(follow.state(), &folded(&EXAMPLE));
    }

    #[test]
    fn follow_unchanged_without_complete_line() {
        let mut follow = Follow::default();
        assert!(follow.read(&joined(&EXAMPLE[..1])));
        let before = follow.state().clone();
        assert!(!follow.read(EXAMPLE[1].as_bytes()));
        assert!(!follow.read(b""));
        assert_eq!(follow.state(), &before);
    }

    #[test]
    fn follow_knows_the_run_finished() {
        let mut follow = Follow::default();
        follow.read(&joined(&EXAMPLE[..3]));
        assert!(!follow.finished());
        follow.read(&joined(&EXAMPLE[3..]));
        assert!(follow.finished());
    }

    /// A writer that takes everything, and is closed when flushed.
    struct Closed;

    impl Write for Closed {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
    }

    #[test]
    fn draw_reports_a_closed_output() {
        let drawn = draw(&mut Closed, &State::default());
        assert_eq!(drawn.map_err(|e| e.kind()), Err(io::ErrorKind::BrokenPipe));
    }

    #[test]
    fn draw_writes_the_state() {
        let mut out = Vec::new();
        draw(&mut out, &folded(&EXAMPLE)).unwrap();
        let text = plain(&out);
        assert!(text.contains("保存"), "{text:?}");
        assert!(text.contains("goal_achieved"), "{text:?}");
    }

    /// The text of terminal output, without the escape sequences. Cells
    /// after a wide character are skipped by cursor moves, which hide it.
    fn plain(bytes: &[u8]) -> String {
        let step = |escaped: &mut bool, c: char| {
            let inside = *escaped || c == '\x1b';
            *escaped = inside && !c.is_ascii_alphabetic();
            Some((!inside).then_some(c))
        };
        let text = String::from_utf8_lossy(bytes);
        text.chars().scan(false, step).flatten().collect()
    }

    #[test]
    fn area_falls_back_when_size_is_unknown() {
        assert_eq!(area(None), Rect::new(0, 0, 80, 24));
        assert_eq!(area(Some((120, 40))), Rect::new(0, 0, 120, 40));
        for zero in [(0, 24), (80, 0), (0, 0)] {
            assert_eq!(area(Some(zero)), Rect::new(0, 0, 80, 24), "{zero:?}");
        }
    }
}
