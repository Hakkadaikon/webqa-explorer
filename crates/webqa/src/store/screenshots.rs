//! The screenshots of the issues: where they go and what they are called.

#[cfg(test)]
mod tests;

use serde_json::Value;
use webqa_core::issue::IssueKind;

/// The directory of the screenshots inside a run's directory.
pub const DIR: &str = "screenshots";

/// The file name of the screenshot of an issue of `kind` found at `step`.
#[must_use]
pub fn name(kind: IssueKind, step: u64) -> String {
    format!("ISSUE_{}_step{step}.png", kind_name(kind))
}

/// The name of `kind`, as serde writes it.
fn kind_name(kind: IssueKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .as_ref()
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}
