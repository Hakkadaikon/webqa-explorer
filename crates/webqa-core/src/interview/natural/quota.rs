//! The cap shared by the mismatch and naturalness questions.

use std::num::NonZeroU32;

use super::{Placed, place};
use crate::interview::ask::MismatchQuestion;
use crate::interview::issue::{Issue, UncheckedPurpose, rank, unchecked};

/// What is left of the cap on the questions asked after the trials.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quota {
    left: usize,
}

impl Quota {
    /// The whole cap, `interview.max_natural_questions`.
    #[must_use]
    pub fn new(cap: NonZeroU32) -> Self {
        Self {
            left: usize::try_from(cap.get()).unwrap_or(usize::MAX),
        }
    }

    /// Nothing left: every question goes over the cap, unasked. For a run
    /// whose judge stopped answering.
    #[must_use]
    pub const fn spent() -> Self {
        Self { left: 0 }
    }

    /// How many questions are left.
    #[must_use]
    pub const fn left(self) -> usize {
        self.left
    }

    /// Splits `items` into those asked, the first ones up to what is
    /// left, and the rest; uses up what it takes.
    #[must_use]
    pub fn take<T>(&mut self, mut items: Vec<T>) -> (Vec<T>, Vec<T>) {
        let at = self.left.min(items.len());
        let rest = items.split_off(at);
        self.left -= at;
        (items, rest)
    }
}

/// The mismatch questions to ask, in the order the screen purposes were
/// written, and the screen purposes over the cap. Call it first.
#[must_use]
pub fn mismatch_round<'a>(
    quota: &mut Quota,
    questions: Vec<MismatchQuestion<'a>>,
) -> (Vec<MismatchQuestion<'a>>, Vec<UncheckedPurpose>) {
    let (asked, over) = quota.take(questions);
    (asked, over.iter().map(unchecked).collect())
}

/// The issues to ask about, in the ledger order, and those over the cap,
/// placed as unchecked. Call it after the mismatch questions are answered,
/// with the mismatch issues among `issues`.
#[must_use]
pub fn natural_round(quota: &mut Quota, mut issues: Vec<Issue>) -> (Vec<Issue>, Vec<Placed>) {
    issues.sort_by(rank);
    let (asked, over) = quota.take(issues);
    let over = over.into_iter().map(|issue| place(issue, None)).collect();
    (asked, over)
}
