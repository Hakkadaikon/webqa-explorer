//! What the viewer has read of one run, and where to draw it.

use std::mem::take;

use ratatui::layout::Rect;
use webqa_core::event::Line;

use crate::tail::Tail;
use crate::view::State;

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

/// The area to draw in: the terminal's size, or 80×24 when unknown.
#[must_use]
pub fn area(size: Option<(u16, u16)>) -> Rect {
    let (width, height) = size.unwrap_or((80, 24));
    Rect::new(0, 0, width, height)
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

    #[test]
    fn area_falls_back_when_size_is_unknown() {
        assert_eq!(area(None), Rect::new(0, 0, 80, 24));
        assert_eq!(area(Some((120, 40))), Rect::new(0, 0, 120, 40));
    }
}
