//! The one-line instruction to fix an issue, from fixed sentences.

use super::wire::quoted;
use crate::interview::issue::{Detail, Issue, Place, Target};

/// What makes a character break a line: control characters and the
/// Unicode line and paragraph separators.
const BREAKS: [fn(char) -> bool; 2] = [char::is_control, is_separator];

/// The one-line instruction to fix `issue`.
#[must_use]
pub fn instruction(issue: &Issue) -> String {
    let at = place(&issue.place);
    let about = issue.target.as_ref().map_or_else(String::new, target);
    flat(&format!("{at}で、{about}{}", body(&issue.detail)))
}

/// Where the issue happened.
fn place(place: &Place) -> String {
    match place {
        Place::Screen { url, .. } => format!("{url} の画面"),
        Place::Pattern(pattern) => format!("URL の型 {pattern} の画面"),
    }
}

/// The element the issue is about.
fn target(target: &Target) -> String {
    format!("{} {} について、", target.role, quoted(&target.name))
}

/// What to do, by kind.
fn body(detail: &Detail) -> String {
    match detail {
        Detail::Hesitant { expectation, .. } => {
            let expected = expected(expectation);
            format!("このペルソナが迷わずに次の操作を選べるようにする{expected}")
        }
        Detail::Visibility { reason, .. } => {
            format!("このペルソナに見えやすく、目立つようにする(理由: {reason})")
        }
        Detail::Unreached { expectation, .. } => {
            let expected = expectation.as_deref().map_or_else(String::new, expected);
            format!("このペルソナが目的まで進めるようにする{expected}")
        }
        Detail::Mismatch { purpose, .. } => {
            format!("書いた目的「{purpose}」が利用者に伝わるようにする")
        }
    }
}

/// What the persona expected, in parentheses; nothing when it was not asked,
/// since no operation was carried out at the step.
fn expected(expectation: &str) -> String {
    if expectation.is_empty() {
        String::new()
    } else {
        format!("(期待: {expectation})")
    }
}

/// `text` with each character that breaks a line made a space.
fn flat(text: &str) -> String {
    text.chars().map(unbroken).collect()
}

/// `c`, or a space when it breaks a line.
fn unbroken(c: char) -> char {
    let breaks = BREAKS.iter().any(|test| test(c));
    if breaks { ' ' } else { c }
}

/// Whether `c` is the Unicode line or paragraph separator.
fn is_separator(c: char) -> bool {
    ['\u{2028}', '\u{2029}'].contains(&c)
}
