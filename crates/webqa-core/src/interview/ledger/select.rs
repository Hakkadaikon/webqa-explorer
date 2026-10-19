//! Setting the selection of an issue in a ledger text, changing only the
//! one character of its checkbox.

use super::{LedgerError, read};
use crate::interview::issue::Number;

/// The checkbox characters a ledger may hold.
const BOXES: [char; 3] = [' ', 'x', 'X'];

/// The start of a first line, before the checkbox character.
const OPEN: &str = "- [";

/// `text` with only the checkbox of `number` set to `selected`.
///
/// # Errors
///
/// [`LedgerError::Malformed`] when `text` cannot be read, and
/// [`LedgerError::NoIssue`] when no issue has the number.
pub fn set_selection(text: &str, number: Number, selected: bool) -> Result<String, LedgerError> {
    read(text)?;
    let mark = if selected { 'x' } else { ' ' };
    let set: Vec<Option<String>> = text
        .split_inclusive('\n')
        .map(|line| checked(line, number, mark))
        .collect();
    let found = set.iter().any(Option::is_some);
    let out = text.split_inclusive('\n').zip(set).map(kept).collect();
    found.then_some(out).ok_or(LedgerError::NoIssue(number))
}

/// The line as set, or as it was.
fn kept((line, set): (&str, Option<String>)) -> String {
    set.unwrap_or_else(|| line.to_owned())
}

/// `line` with its checkbox set to `mark`, when it is the first line of
/// the issue `number`.
fn checked(line: &str, number: Number, mark: char) -> Option<String> {
    let rest = line
        .strip_prefix(OPEN)
        .and_then(|rest| rest.strip_prefix(BOXES))?;
    let named = rest
        .strip_prefix(&format!("] {number}"))
        .is_some_and(ends_number);
    named.then(|| format!("{OPEN}{mark}{rest}"))
}

/// Whether the number is followed by a space or the end of the line.
fn ends_number(after: &str) -> bool {
    after.chars().next().is_none_or(is_end)
}

/// Whether `c` ends the number: a space or a line ending.
fn is_end(c: char) -> bool {
    [' ', '\n', '\r'].contains(&c)
}

#[cfg(test)]
pub(super) mod tests;
