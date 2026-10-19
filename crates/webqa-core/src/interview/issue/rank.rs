//! The order of the issues in the ledger.

use std::cmp::Ordering;

use super::Issue;

/// The ledger order: the higher rate k/N first, then the lower number.
#[must_use]
pub fn rank(a: &Issue, b: &Issue) -> Ordering {
    let left = b.count.occurred * a.count.finished;
    let right = a.count.occurred * b.count.finished;
    left.cmp(&right).then_with(|| a.number.cmp(&b.number))
}
