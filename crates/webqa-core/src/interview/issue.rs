//! The issues an interview finds: their kinds, numbers, and how often
//! they happened over the finished trials.

mod find;
mod gather;
mod label;
mod mismatch;
mod number;
mod rank;
mod record;

use std::fmt;

pub use gather::step_issues;
pub use label::{CLUES, HALTS, KINDS, clue_label};
pub use mismatch::{Ruling, ruling, unchecked};
pub use number::{Key, Number, ScreenMark, Spot};
pub use rank::rank;
pub use record::{Ran, Record, Run, Seen, visits};

use crate::config::raw::Op;
use crate::interview::clue::Clue;
use crate::screen::Rect;

/// What kind of issue an interview found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The persona hesitated (a clue of the step).
    Hesitant,
    /// The judge listed an element as hard to see.
    Visibility,
    /// A finished trial got stuck or ran out of budget.
    Unreached,
    /// The judge said the guessed purposes differ from the written one.
    Mismatch,
}

/// An issue of an interview run, gathered over the finished trials.
/// `Debug` shows only the number, the kind and the count.
#[derive(Clone, PartialEq)]
pub struct Issue {
    /// The number that stays the same across runs.
    pub number: Number,
    /// Where it happened.
    pub place: Place,
    /// The element it is about. `None` for issues without an element.
    pub target: Option<Target>,
    /// What the judge said, by kind.
    pub detail: Detail,
    /// How often it happened.
    pub count: Count,
    /// The first occurrence: its screenshot is the issue's.
    pub shot: Option<Shot>,
    /// What ran on the page before the first occurrence, in that trial.
    pub path: Vec<Act>,
    /// The confidence of the operation chosen at the first occurrence.
    /// Only hesitation issues have one.
    pub confidence: Option<f64>,
}

/// Where an issue happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// A screen: its URL at the first occurrence (already redacted, as
    /// the screen was) and the mark of its identity.
    Screen {
        /// The URL as a string.
        url: String,
        /// The mark of the screen's identity.
        mark: ScreenMark,
    },
    /// The URL pattern of a screen purpose, as written.
    Pattern(String),
}

/// An element an issue is about. `Debug` hides the name.
#[derive(Clone, PartialEq, Eq)]
pub struct Target {
    /// ARIA role.
    pub role: String,
    /// The accessible name, redacted but not normalized.
    pub name: String,
    /// Its position among the page's elements with the same role and
    /// normalized name, from 0 (the `nth` of its
    /// [`ElementId`](crate::screen::ElementId)).
    pub nth: usize,
    /// Its box, if the page reported one.
    pub rect: Option<Rect>,
}

/// What the judge said about an issue, by kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Detail {
    /// The clues of the first occurrence, and what the persona expected.
    Hesitant {
        /// The clues of the step.
        clues: Vec<Clue>,
        /// What the persona expected.
        expectation: String,
    },
    /// Why the element was hard to see, and what the persona expected.
    Visibility {
        /// Why, as the judge wrote it.
        reason: String,
        /// What the persona expected.
        expectation: String,
    },
    /// How the trial stopped, and what the persona expected at its last
    /// step. `None` when that step's answers were unusable.
    Unreached {
        /// How the trial stopped.
        halt: Halt,
        /// What the persona expected at the last step.
        expectation: Option<String>,
    },
    /// The written purpose and the guesses passed to the mismatch question.
    Mismatch {
        /// The written purpose.
        purpose: String,
        /// The guesses passed to the question.
        guesses: Vec<String>,
    },
}

/// How a trial that did not reach the purpose stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Halt {
    /// The judge chose stuck: the persona gave up.
    GaveUp,
    /// Steps without progress reached the stuck threshold.
    NoProgress,
    /// The decision budget ran out.
    BudgetSpent,
}

/// How often an issue happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Count {
    /// k: the finished trials it happened in.
    pub occurred: usize,
    /// N: the finished trials. For a mismatch, those that reached the pattern.
    pub finished: usize,
    /// The finished trials that reached its screen (or pattern).
    pub reached: usize,
}

/// A step of a trial, numbered from 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shot {
    /// The trial's number among the trials started, from 1.
    pub trial: usize,
    /// The step's number in the trial, from 1.
    pub step: usize,
}

/// An operation that ran on the page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Act {
    /// The operation kind.
    pub op: Op,
    /// The element, for operations on one.
    pub target: Option<Target>,
}

/// A screen purpose whose mismatch was not checked: its answer was
/// unusable, or the cap on questions was used up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UncheckedPurpose {
    /// The URL pattern, as written.
    pub pattern: String,
    /// The written purpose.
    pub purpose: String,
}

impl Issue {
    /// The kind, from the detail.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.detail.kind()
    }
}

impl Detail {
    /// The kind of issue this detail belongs to.
    const fn kind(&self) -> Kind {
        match self {
            Self::Hesitant { .. } => Kind::Hesitant,
            Self::Visibility { .. } => Kind::Visibility,
            Self::Unreached { .. } => Kind::Unreached,
            Self::Mismatch { .. } => Kind::Mismatch,
        }
    }
}

impl fmt::Debug for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Issue")
            .field("number", &format_args!("{}", self.number))
            .field("kind", &self.kind())
            .field("count", &self.count)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Target")
            .field("role", &self.role)
            .field("name", &"…")
            .field("nth", &self.nth)
            .field("rect", &self.rect)
            .finish()
    }
}

#[cfg(test)]
pub(crate) mod tests;
