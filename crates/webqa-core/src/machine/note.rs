//! What the core records during a run, for the run loop to log.

use super::Seen;
use crate::issue::{Issue, IssueKind};
use crate::judge::{JudgeError, Unusable};

/// One record. The notes only grow; the run loop reads the new ones.
#[derive(Debug, Clone, PartialEq)]
pub enum Note {
    /// A proposal the guardrails stopped.
    Rejected(Rejection),
    /// An answer that could not be used.
    Anomaly(Anomaly),
    /// An issue the core found: leaving the allowed origins.
    Issue(Issue),
}

/// Why a proposal was stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    /// A forbidden element, or one without material.
    Forbidden,
    /// A "back" the judge is not sure of.
    LowConfidence,
    /// The element is gone, or has no reference to look it up by.
    Missing,
    /// Another element took its place.
    Replaced,
}

/// What was wrong with the answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Anomaly {
    /// The judge failed.
    Judge(JudgeError),
    /// The answers arrived but cannot be used.
    Unusable(Unusable),
}

/// The issue of landing on `seen`, outside the allowed origins.
pub(super) fn left(seen: &Seen) -> Note {
    Note::Issue(Issue {
        kind: IssueKind::OutsideDomain,
        evidence: Some(seen.screen.map(|s| s.url.to_string())),
        label: None,
        p: None,
    })
}
