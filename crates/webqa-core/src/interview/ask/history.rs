//! The recent operations of a trial, added once to the screen text.

use crate::redact::Redacted;
use crate::screen::{ElementId, ScreenText, escaped};

/// How many of the latest operations the judge is told.
pub const RECENT: usize = 5;

/// One operation of the trial, as the judge is told it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Past {
    /// The operation, as the browser knows it.
    pub op: String,
    /// The element operated on, if any. Its name is redacted.
    pub target: Option<ElementId>,
    /// Whether the screen changed by it.
    pub changed: bool,
}

/// The section of the latest [`RECENT`] of `past`; none when it is empty.
///
/// Oldest first, one line each. The values typed and the text of the page
/// are not in a line. It is not redacted yet: pass it through the redactor
/// before [`with_section`].
#[must_use]
pub fn recent_section(past: &[Past]) -> Option<String> {
    let from = past.len().saturating_sub(RECENT);
    let lines: Vec<String> = past
        .iter()
        .skip(from)
        .zip(1..)
        .map(|(done, n)| line(n, done))
        .collect();
    (!lines.is_empty()).then(|| format!("直近の操作\n{}", lines.join("\n")))
}

/// `screen` followed by `section`.
#[must_use]
pub fn with_section(
    screen: &Redacted<ScreenText>,
    section: &Redacted<String>,
) -> Redacted<ScreenText> {
    screen.map(|text| text.followed_by(section.as_inner()))
}

fn line(n: usize, done: &Past) -> String {
    let target = done.target.as_ref().map_or_else(
        || "なし".to_owned(),
        |id| format!("「{}」", escaped(&id.name)),
    );
    let result = if done.changed {
        "画面が変わった"
    } else {
        "画面は変わらなかった"
    };
    format!("{n}. {} {target} → {result}", done.op)
}
