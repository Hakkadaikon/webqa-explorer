//! The recent operations of a trial, added once to the screen text.

use crate::machine::Rejection;
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
    /// What came of it.
    pub after: After,
}

/// What came of an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum After {
    /// The URL changed.
    Moved,
    /// The URL stayed, and the text in the viewport changed: a scroll
    /// shows this.
    Shown,
    /// Neither changed.
    Unchanged,
    /// The guardrails stopped it, so it was not carried out.
    Stopped(Rejection),
}

/// The section of the latest [`RECENT`] of `past`; none when it is empty.
///
/// Oldest first, one line each. When the last operation was the same as the
/// ones before it, a last line says how many times in a row, counting every
/// operation of `past`. The values typed and the text of the page are not
/// in a line. It is not redacted yet: pass it through the redactor before
/// [`with_section`].
#[must_use]
pub fn recent_section(past: &[Past]) -> Option<String> {
    let from = past.len().saturating_sub(RECENT);
    let mut lines: Vec<String> = past
        .iter()
        .skip(from)
        .zip(1..)
        .map(|(done, n)| line(n, done))
        .collect();
    lines.extend(repeated(past));
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
    format!("{n}. {} {} → {}", done.op, target(done), result(done.after))
}

/// The element of `done` as a line names it.
fn target(done: &Past) -> String {
    done.target.as_ref().map_or_else(
        || "なし".to_owned(),
        |id| format!("「{}」", escaped(&id.name)),
    )
}

fn result(after: After) -> String {
    match after {
        After::Moved => "別のページに移った".to_owned(),
        After::Shown => "同じページで表示が変わった".to_owned(),
        After::Unchanged => "画面は変わらなかった".to_owned(),
        After::Stopped(why) => format!("止められた({})", reason(why)),
    }
}

const fn reason(why: Rejection) -> &'static str {
    match why {
        Rejection::Forbidden => "禁止の要素",
        Rejection::LowConfidence => "確信度が低い",
        Rejection::Missing => "要素が見つからない",
        Rejection::Replaced => "別の要素に替わった",
    }
}

/// The line of how many times in a row the last operation of `past` was
/// done on the same element; none when it was done once.
fn repeated(past: &[Past]) -> Option<String> {
    let (last, before) = past.split_last()?;
    let same = |done: &&Past| done.op == last.op && done.target == last.target;
    let times = before
        .iter()
        .rev()
        .take_while(same)
        .count()
        .saturating_add(1);
    let line = format!(
        "直前の操作 {} {} が{times}回続いている",
        last.op,
        target(last)
    );
    (times > 1).then_some(line)
}
