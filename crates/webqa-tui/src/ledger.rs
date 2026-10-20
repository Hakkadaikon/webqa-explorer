//! The ledger screen's state, folded from inputs one at a time. It never
//! touches the file.

mod detail;
pub mod render;
#[cfg(test)]
mod tests;

use webqa_core::interview::issue::{Kind, Number};
use webqa_core::interview::ledger::{Entry, Ledger};

/// What the ledger screen receives.
#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    /// Move the cursor up one issue.
    Up,
    /// Move the cursor down one issue.
    Down,
    /// Show the issue under the cursor.
    Open,
    /// Go back to the list.
    Close,
    /// Show only issues of this kind, or all with `None`.
    Filter(Option<Kind>),
    /// The ledger after a write back.
    Saved(Ledger),
    /// The write back was refused: the file changed after it was opened.
    ChangedOutside,
    /// The write back failed for another reason: nothing was written.
    WriteFailed,
}

/// A notice shown under the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notice {
    /// The file changed outside after it was opened; nothing was written.
    ChangedOutside,
    /// The write back failed; nothing was written.
    WriteFailed,
}

/// The ledger screen.
#[derive(Debug, Clone, PartialEq)]
pub struct Screen {
    /// The ledger as last read or written.
    pub ledger: Ledger,
    /// The kind shown, or every kind.
    pub filter: Option<Kind>,
    /// The position of the cursor among the shown issues, from 0.
    pub cursor: usize,
    /// Whether the issue under the cursor is open.
    pub open: bool,
    /// The notice, if any.
    pub notice: Option<Notice>,
}

impl Screen {
    /// A list of every issue, the cursor on the first.
    #[must_use]
    pub const fn new(ledger: Ledger) -> Self {
        Self {
            ledger,
            filter: None,
            cursor: 0,
            open: false,
            notice: None,
        }
    }

    /// Applies one input.
    #[must_use]
    pub fn fold(self, input: Input) -> Self {
        match input {
            Input::Filter(filter) => Self {
                filter,
                cursor: 0,
                open: false,
                ..self
            },
            Input::Saved(ledger) => Self {
                ledger,
                notice: None,
                ..self
            }
            .clamped(),
            Input::ChangedOutside | Input::WriteFailed => self.told(&input),
            moving @ (Input::Up | Input::Down | Input::Open | Input::Close) => self.moved(&moving),
        }
    }

    /// The issues shown, in ledger order.
    #[must_use]
    pub fn shown(&self) -> Vec<&Entry> {
        let all = self.ledger.entries.iter();
        all.filter(|entry| self.filter.is_none_or(|kind| entry.issue.kind() == kind))
            .collect()
    }

    /// The issue under the cursor, if any.
    #[must_use]
    pub fn current(&self) -> Option<&Entry> {
        self.shown().into_iter().nth(self.cursor)
    }

    /// What toggling the issue under the cursor writes: its number and
    /// the new value. `None` when no issue is shown.
    #[must_use]
    pub fn toggle(&self) -> Option<(Number, bool)> {
        self.current()
            .map(|entry| (entry.issue.number, !entry.selected))
    }

    /// The position of the last shown issue; 0 when none is shown.
    fn last(&self) -> usize {
        self.shown().len().saturating_sub(1)
    }

    /// Shows the notice of a refused or failed write back.
    fn told(self, input: &Input) -> Self {
        let notice = if *input == Input::ChangedOutside {
            Notice::ChangedOutside
        } else {
            Notice::WriteFailed
        };
        Self {
            notice: Some(notice),
            ..self
        }
    }

    fn clamped(self) -> Self {
        Self {
            cursor: self.cursor.min(self.last()),
            ..self
        }
    }

    /// Applies a cursor input.
    fn moved(self, input: &Input) -> Self {
        match input {
            Input::Up => Self {
                cursor: self.cursor.saturating_sub(1),
                ..self
            },
            Input::Down => Self {
                cursor: (self.cursor + 1).min(self.last()),
                ..self
            },
            Input::Open => Self { open: true, ..self },
            // Only Close arrives here; `fold` handles the others itself.
            Input::Close
            | Input::Filter(_)
            | Input::Saved(_)
            | Input::ChangedOutside
            | Input::WriteFailed => Self {
                open: false,
                ..self
            },
        }
    }
}

/// The notice's text.
#[must_use]
pub const fn notice_text(notice: Notice) -> &'static str {
    match notice {
        Notice::ChangedOutside => {
            "台帳が外で書き換えられたので、書き戻さなかった。開き直してから選び直す"
        }
        Notice::WriteFailed => "台帳を書き戻せなかった。ファイルを確かめる",
    }
}
