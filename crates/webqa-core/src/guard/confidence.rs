//! Low confidence "back": going back is only done when the judge is sure.

#[cfg(test)]
mod tests;

use std::cmp::Ordering;

/// The largest finite probability, or `None` when there is none.
///
/// The range is checked with the judge's answer, not here.
#[must_use]
pub fn confidence(probabilities: &[f64]) -> Option<f64> {
    probabilities
        .iter()
        .copied()
        .filter(|p| p.is_finite())
        .reduce(f64::max)
}

/// Whether a "back" with this confidence must not run: no confidence, below
/// `threshold` (`limits.back_confidence`), or not comparable (NaN on either
/// side). The threshold itself is not low.
#[must_use]
pub fn is_low(confidence: Option<f64>, threshold: f64) -> bool {
    confidence.is_none_or(|c| c.partial_cmp(&threshold).is_none_or(Ordering::is_lt))
}
