//! The recent operations of a trial, added once to the screen text.

use crate::redact::Redacted;
use crate::screen::{ElementId, ScreenText};

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

/// `screen` followed by the latest [`RECENT`] of `past`, oldest first, one
/// line each. `screen` as it is when `past` is empty. The values typed and
/// the text of the page are not in a line.
#[must_use]
pub fn with_recent(screen: &Redacted<ScreenText>, past: &[Past]) -> Redacted<ScreenText> {
    if past.is_empty() {
        return screen.clone();
    }
    let from = past.len().saturating_sub(RECENT);
    let lines: Vec<String> = past
        .iter()
        .skip(from)
        .zip(1..)
        .map(|(done, n)| line(n, done))
        .collect();
    let section = format!("直近の操作\n{}", lines.join("\n"));
    screen.map(|text| text.followed_by(&section))
}

fn line(n: usize, done: &Past) -> String {
    let target = done
        .target
        .as_ref()
        .map_or_else(|| "なし".to_owned(), |id| format!("「{}」", id.name));
    let result = if done.changed {
        "画面が変わった"
    } else {
        "画面は変わらなかった"
    };
    format!("{n}. {} {target} → {result}", done.op)
}
