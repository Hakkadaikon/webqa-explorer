//! The issues of a ledger that a score counts.

#[cfg(test)]
mod tests;

use url::Url;
use webqa_core::interview::issue::Place;
use webqa_core::interview::ledger::{Entry, read};
use webqa_core::interview::natural::Section;

use crate::EvalError;
use crate::truth::{Element, KindId};

/// An issue of a ledger, as a score sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// The issue number as the ledger shows it.
    pub number: String,
    /// The kind.
    pub kind: KindId,
    /// The screen's path, or the URL pattern of a mismatch.
    pub path: String,
    /// The element, when the issue has one.
    pub element: Option<Element>,
}

/// The issues that are in the body or not checked, in the ledger's order.
///
/// # Errors
///
/// [`EvalError::Ledger`] when the text is not a ledger.
pub fn found(ledger_text: &str) -> Result<Vec<Found>, EvalError> {
    let ledger = read(ledger_text).map_err(|error| EvalError::Ledger(error.to_string()))?;
    Ok(ledger
        .entries
        .iter()
        .filter(|entry| entry.section != Section::OffPersona)
        .map(from_entry)
        .collect())
}

fn from_entry(entry: &Entry) -> Found {
    let issue = &entry.issue;
    Found {
        number: issue.number.to_string(),
        kind: issue.kind().into(),
        path: path(&issue.place),
        element: issue.target.as_ref().map(|target| Element {
            role: target.role.clone(),
            name: target.name.clone(),
        }),
    }
}

fn path(place: &Place) -> String {
    match place {
        Place::Screen { url, .. } => {
            Url::parse(url).map_or_else(|_| url.clone(), |u| u.path().to_owned())
        }
        Place::Pattern(pattern) => pattern.clone(),
    }
}
