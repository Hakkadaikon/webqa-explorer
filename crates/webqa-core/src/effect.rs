//! What the core asks the run loop to do, nested two kinds at a level.

use std::time::Duration;

use crate::judge::Question;
use crate::redact::Redacted;
use crate::screen::{ElementId, ScreenText};

/// One thing for the run loop to do.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Outside the page: the judge model or the clock.
    Outside(Outside),
    /// On the page.
    Page(PageEffect),
}

/// An effect outside the page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outside {
    /// Ask the judge model.
    Ask(Questions),
    /// Wait.
    Sleep(Wake),
}

/// An effect on the page.
#[derive(Debug, Clone, PartialEq)]
pub enum PageEffect {
    /// Look up the elements again just before touching one.
    Look(Target),
    /// Operate the browser.
    Touch(Touch),
}

/// What to send to the judge model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Questions {
    /// The screen text, already redacted.
    pub screen: Redacted<ScreenText>,
    /// The questions to answer.
    pub questions: Vec<Question>,
}

/// When to wake up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wake {
    /// Time since the run started.
    pub at: Duration,
}

/// The element to look up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The element.
    pub element: ElementId,
}

/// A browser operation.
#[derive(Debug, Clone, PartialEq)]
pub struct Touch {
    /// The kind of operation, such as `click`.
    pub op: String,
    /// The element to operate on, if the operation needs one.
    pub target: Option<ElementId>,
    /// The probability the judge model gave the operation.
    pub confidence: f64,
}
