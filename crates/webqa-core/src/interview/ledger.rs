//! The interview ledger: written as one Markdown text, read back, and its
//! checkboxes set without touching the other bytes.

mod goal;
mod instruction;
mod label;
mod mask;
mod read;
mod select;
mod wire;
mod write;

pub use goal::inferred_goal;
pub use instruction::instruction;
pub use read::read;
pub use select::set_selection;
pub use write::write;

use crate::interview::issue::{Issue, Number, UncheckedPurpose, rank};
use crate::interview::natural::{PersonaTrait, Placed, Section};

/// An interview ledger: its head, its issues and the unchecked screen
/// purposes.
#[derive(Debug, Clone, PartialEq)]
pub struct Ledger {
    /// The head.
    pub head: Head,
    /// The issues, each with its section. Written in the ledger order:
    /// by section, then by [`rank`](crate::interview::issue::rank).
    pub entries: Vec<Entry>,
    /// The screen purposes whose mismatch was not checked, in order.
    pub unchecked_purposes: Vec<UncheckedPurpose>,
}

/// The head of a ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Head {
    /// The persona's name.
    pub persona: String,
    /// The purpose.
    pub goal: Goal,
    /// The start URL.
    pub start: String,
    /// N: the finished trials.
    pub finished: usize,
    /// Whether the judge declared that it reads images, so visibility
    /// was checked.
    pub images: bool,
}

/// The purpose of the run, as the ledger records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Goal {
    /// The written purpose's goal.
    Written(String),
    /// Inferred: the purposes the judge guessed at the start screen.
    Inferred(Vec<String>),
}

/// An issue in the ledger.
///
/// A mismatch issue is placed at a URL pattern, and every other kind at a
/// screen; the ledger cannot be read back otherwise.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// Whether the user selected it to fix.
    pub selected: bool,
    /// Its section.
    pub section: Section,
    /// The issue.
    pub issue: Issue,
    /// The trait of the persona it came from. `None` when unchecked.
    pub persona_trait: Option<PersonaTrait>,
    /// How it compares with the previous run.
    pub compared: Compared,
}

/// How an issue compares with the previous run's ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compared {
    /// Not in the previous ledger, or no previous ledger was given.
    New,
    /// Also in the previous ledger.
    Remaining,
    /// In the previous ledger only, and its screen was reached at least
    /// as often as then.
    Resolved,
    /// In the previous ledger only, and its screen was reached less often
    /// than then, or not at all.
    Unverified,
}

/// A ledger text that cannot be read. Carries no text of the ledger.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LedgerError {
    /// A line is not in the fixed form, or one is missing.
    #[error("台帳の {line} 行目が決まった書式でない")]
    Malformed {
        /// The line, from 1. One past the last line when the text ends early.
        line: usize,
    },
    /// No issue has the number.
    #[error("台帳に課題 {0} が無い")]
    NoIssue(Number),
}

impl From<Placed> for Entry {
    fn from(placed: Placed) -> Self {
        Self {
            selected: false,
            section: placed.section,
            issue: placed.issue,
            persona_trait: placed.persona_trait,
            compared: Compared::New,
        }
    }
}

impl Ledger {
    /// Sorts the entries into the ledger order (stable).
    pub fn arrange(&mut self) {
        self.entries.sort_by(|a, b| {
            a.section
                .cmp(&b.section)
                .then_with(|| rank(&a.issue, &b.issue))
        });
    }

    /// The numbers of the selected issues, in ledger order.
    #[must_use]
    pub fn selected(&self) -> Vec<Number> {
        self.entries
            .iter()
            .filter(|entry| entry.selected)
            .map(|entry| entry.issue.number)
            .collect()
    }
}

#[cfg(test)]
pub(crate) mod tests;
