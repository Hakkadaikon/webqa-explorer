//! The answer key (planted issues) and the human labels of other issues.

#[cfg(test)]
mod tests;

use serde::Deserialize;
use webqa_core::interview::issue::Kind;

use crate::EvalError;

/// The kind of an issue, as the files write it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KindId {
    /// The persona hesitated.
    Hesitant,
    /// An element is hard to see.
    Visibility,
    /// A trial got stuck.
    Unreached,
    /// The screen's purpose differs from the written one.
    Mismatch,
}

impl From<Kind> for KindId {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Hesitant => Self::Hesitant,
            Kind::Visibility => Self::Visibility,
            Kind::Unreached => Self::Unreached,
            Kind::Mismatch => Self::Mismatch,
        }
    }
}

/// An element, by its role and name.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Element {
    /// The role.
    pub role: String,
    /// The name.
    pub name: String,
}

/// A planted issue.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plant {
    /// The name of the plant.
    pub id: String,
    /// The kind.
    pub kind: KindId,
    /// The screen's path, or the URL pattern of a mismatch. `None` matches any.
    pub path: Option<String>,
    /// The element. `None` matches any.
    pub element: Option<Element>,
    /// What was planted.
    pub note: String,
}

/// A human's ruling on an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// A real usability problem.
    Real,
    /// A mistake.
    False,
}

/// A ruling on the issues of a kind, path and element.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Label {
    /// The kind.
    pub kind: KindId,
    /// The path. `None` matches any.
    pub path: Option<String>,
    /// The element. `None` matches any.
    pub element: Option<Element>,
    /// The ruling.
    pub verdict: Verdict,
}

/// Reads the answer key.
///
/// # Errors
///
/// [`EvalError::Yaml`] when the text is not a list of plants.
pub fn read_truth(text: &str) -> Result<Vec<Plant>, EvalError> {
    read_list(text)
}

/// Reads the human labels. An empty document is no labels.
///
/// # Errors
///
/// [`EvalError::Yaml`] when the text is not a list of labels.
pub fn read_labels(text: &str) -> Result<Vec<Label>, EvalError> {
    read_list(text)
}

fn read_list<T: for<'de> Deserialize<'de>>(text: &str) -> Result<Vec<T>, EvalError> {
    if text.trim().is_empty() {
        return Ok(vec![]);
    }
    serde_saphyr::from_str(text).map_err(|error| EvalError::Yaml(error.to_string()))
}
