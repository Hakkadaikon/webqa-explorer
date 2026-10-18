//! Issues found on a page: their kinds, classes and detection.

mod debug;
mod name;
#[cfg(test)]
mod tests;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::redact::Redacted;
use crate::screen::{Element, Screen};

pub use name::is_placeholder_name;

/// What an issue is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueKind {
    /// The page shows an error, by the judge's confidence.
    ErrorDisplay,
    /// An element is named after an image file or a placeholder.
    AccessibleName,
    /// The browser console reported an error.
    ConsoleError,
    /// The page moved outside the allowed domains.
    OutsideDomain,
    /// A finding that only the judge can make.
    Semantic,
}

impl IssueKind {
    /// Whether issues of this kind count towards the exit code.
    #[must_use]
    pub fn class(self) -> IssueClass {
        if self == Self::Semantic {
            IssueClass::Reference
        } else {
            IssueClass::Confirmed
        }
    }
}

/// Whether an issue counts towards the exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueClass {
    /// Counts towards the exit code.
    Confirmed,
    /// For reference only; does not count.
    Reference,
}

/// One issue found from one step's material.
///
/// `Debug` hides the evidence.
#[derive(Clone, PartialEq)]
pub struct Issue {
    /// What the issue is about.
    pub kind: IssueKind,
    /// The page string the issue points at, redacted. `None` for judge-based kinds.
    pub evidence: Option<Redacted<String>>,
    /// The fixed name of a semantic finding. `None` for other kinds.
    pub label: Option<String>,
    /// The judge's confidence. `None` for kinds the core or browser detects.
    pub p: Option<f64>,
}

impl Issue {
    /// The class of the issue's kind.
    #[must_use]
    pub fn class(&self) -> IssueClass {
        self.kind.class()
    }
}

/// A semantic finding answered by the judge.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    /// The fixed name of the finding, chosen by the question; no page text.
    pub label: String,
    /// The probability the judge gave to "present". `None` when missing.
    pub p: Option<f64>,
}

/// An error display issue when the judge's confidence reaches `threshold`.
///
/// A missing or `NaN` confidence records nothing.
#[must_use]
pub fn error_display_issue(p: Option<f64>, threshold: f64) -> Option<Issue> {
    reached(p, threshold).map(|x| Issue {
        kind: IssueKind::ErrorDisplay,
        evidence: None,
        label: None,
        p: Some(x),
    })
}

/// One issue per element named after an image file or a placeholder, in page order.
///
/// Every element of the page counts, in the viewport or not.
#[must_use]
pub fn accessible_name_issues(screen: &Redacted<Screen>) -> Vec<Issue> {
    screen
        .as_inner()
        .elements
        .iter()
        .filter(|e| is_placeholder_name(&e.name))
        .map(|e| name_issue(screen, e))
        .collect()
}

/// An accessible name issue whose evidence is the name as the page shows it.
fn name_issue(screen: &Redacted<Screen>, element: &Element) -> Issue {
    Issue {
        kind: IssueKind::AccessibleName,
        evidence: Some(screen.map(|_| element.name.clone())),
        label: None,
        p: None,
    }
}

/// One issue per distinct console error message, in the order received.
///
/// A repeated message gives only its first issue.
#[must_use]
pub fn console_issues(errors: &[Redacted<String>]) -> Vec<Issue> {
    let mut seen: HashSet<&str> = HashSet::new();
    errors
        .iter()
        .filter(|e| seen.insert(e.as_inner().as_str()))
        .map(console_issue)
        .collect()
}

fn console_issue(error: &Redacted<String>) -> Issue {
    Issue {
        kind: IssueKind::ConsoleError,
        evidence: Some(error.clone()),
        label: None,
        p: None,
    }
}

/// One reference issue per finding whose confidence reaches `threshold`, in order.
#[must_use]
pub fn semantic_issues(findings: &[Finding], threshold: f64) -> Vec<Issue> {
    findings
        .iter()
        .filter_map(|f| semantic_issue(f, threshold))
        .collect()
}

fn semantic_issue(finding: &Finding, threshold: f64) -> Option<Issue> {
    reached(finding.p, threshold).map(|x| Issue {
        kind: IssueKind::Semantic,
        evidence: None,
        label: Some(finding.label.clone()),
        p: Some(x),
    })
}

/// The confidence when it is present and at least `threshold`.
fn reached(p: Option<f64>, threshold: f64) -> Option<f64> {
    p.filter(|x| *x >= threshold)
}

/// Whether any issue counts towards the exit code.
#[must_use]
pub fn has_confirmed(issues: &[Issue]) -> bool {
    issues.iter().any(|i| i.class() == IssueClass::Confirmed)
}
