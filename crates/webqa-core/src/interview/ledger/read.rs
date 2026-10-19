//! Reading a ledger back: the head, then the sections in their order.
//!
//! The steps are joined with `and_then`, so that a line that cannot be
//! read stops the reading at that line.

mod entry;
mod value;

use value::{At, Field, Lines, decimal, json};

use super::label::{
    CHECKED, EMPTY, FINISHED, GOAL, INFERRED, NO_TRIAL, PERSONA, PURPOSE_LINE, SECTIONS, START,
    TITLE, UNCHECKED_SIGHT, VERSION, WRITTEN, key_of,
};
use super::wire::WirePurpose;
use super::{Goal, Head, Ledger, LedgerError};
use crate::interview::issue::UncheckedPurpose;
use crate::interview::natural::Section;

/// The lines of the head about visibility, and whether it was checked.
const SIGHTS: [(bool, &str); 2] = [(true, CHECKED), (false, UNCHECKED_SIGHT)];

/// The persona's name and the purpose, the start URL and the finished
/// trials.
type Values = ((String, Goal), (String, usize));

/// The start of a section heading.
const HEADING: &str = "## ";

/// Reads a ledger written by [`write`](super::write).
///
/// # Errors
///
/// [`LedgerError::Malformed`] with the first line that is not in the
/// fixed form.
pub fn read(text: &str) -> Result<Ledger, LedgerError> {
    let mut lines = Lines::new(text);
    let head = head(&mut lines)?;
    let mut ledger = Ledger {
        head,
        entries: vec![],
        unchecked_purposes: vec![],
    };
    sections(&mut lines, &mut ledger).map(|()| ledger)
}

/// The head, with the blank after it.
fn head(lines: &mut Lines<'_>) -> Result<Head, LedgerError> {
    preamble(lines)?;
    let ((persona, goal), (start, finished)) = values(lines)?;
    sight(lines).map(|images| Head {
        persona,
        goal,
        start,
        finished,
        images,
    })
}

/// The values of the head: the persona's name and the purpose, the start
/// URL and the finished trials.
fn values(lines: &mut Lines<'_>) -> Result<Values, LedgerError> {
    who(lines).and_then(|who| run(lines).map(|run| (who, run)))
}

/// The title, the blank and the version.
fn preamble(lines: &mut Lines<'_>) -> Result<(), LedgerError> {
    lines
        .exact(TITLE)
        .and_then(|()| lines.exact(""))
        .and_then(|()| lines.exact(VERSION))
}

/// The persona's name and the purpose.
fn who(lines: &mut Lines<'_>) -> Result<(String, Goal), LedgerError> {
    let persona = lines.after(PERSONA).and_then(json::<String>);
    persona.and_then(|persona| lines.after(GOAL).and_then(goal).map(|goal| (persona, goal)))
}

/// The purpose after its prefix.
fn goal((text, line): Field<'_>) -> Result<Goal, LedgerError> {
    let written = text
        .strip_prefix(WRITTEN)
        .and_then(|s| serde_json::from_str(s).ok())
        .map(Goal::Written);
    let inferred = text
        .strip_prefix(INFERRED)
        .and_then(|s| serde_json::from_str(s).ok())
        .map(Goal::Inferred);
    written.or(inferred).at(line)
}

/// The start URL and the finished trials.
fn run(lines: &mut Lines<'_>) -> Result<(String, usize), LedgerError> {
    let start = lines.after(START).and_then(json::<String>);
    start.and_then(|start| lines.after(FINISHED).and_then(finished).map(|n| (start, n)))
}

/// The finished trials after their prefix: [`NO_TRIAL`] or a positive
/// decimal.
fn finished((text, line): Field<'_>) -> Result<usize, LedgerError> {
    let none = (text == NO_TRIAL).then_some(0);
    let some = decimal(text).filter(|&n| n > 0);
    none.or(some).at(line)
}

/// Whether visibility was checked, with the blank after the head.
fn sight(lines: &mut Lines<'_>) -> Result<bool, LedgerError> {
    let images = lines
        .next()
        .and_then(|(text, line)| key_of(&SIGHTS, text).at(line));
    images.and_then(|images| lines.exact("").map(|()| images))
}

/// The sections in their order, into `ledger`.
fn sections(lines: &mut Lines<'_>, ledger: &mut Ledger) -> Result<(), LedgerError> {
    for (section, heading) in SECTIONS {
        lines.exact(heading)?;
        lines
            .exact("")
            .and_then(|()| body(lines, section, ledger))?;
    }
    Ok(())
}

/// What a section holds after its heading: [`EMPTY`] or its items.
fn body(lines: &mut Lines<'_>, section: Section, ledger: &mut Ledger) -> Result<(), LedgerError> {
    let empty = lines.peek() == Some(EMPTY);
    let read = if empty { nothing(lines) } else { Ok(()) };
    read.and_then(|()| items(lines, section, ledger))
}

/// The line of an empty section and its blank.
fn nothing(lines: &mut Lines<'_>) -> Result<(), LedgerError> {
    lines.next().and_then(|_| lines.exact(""))
}

/// The items of `section`: up to the next heading, or for the last
/// section, every line left.
fn items(lines: &mut Lines<'_>, section: Section, ledger: &mut Ledger) -> Result<(), LedgerError> {
    let more = if section == Section::OffPersona {
        any
    } else {
        is_item
    };
    while lines.peek().is_some_and(more) {
        item(lines, section, ledger)?;
    }
    Ok(())
}

/// Whether `line` starts an item rather than the next section.
fn is_item(line: &str) -> bool {
    !line.starts_with(HEADING)
}

/// Every line left after the last section is read as an item.
const fn any(_: &str) -> bool {
    true
}

/// An issue, or an unchecked purpose in the unchecked section, and the
/// blank after it.
fn item(lines: &mut Lines<'_>, section: Section, ledger: &mut Ledger) -> Result<(), LedgerError> {
    let read = if is_purpose(section, lines.peek()) {
        purpose(lines).map(|it| ledger.unchecked_purposes.push(it))
    } else {
        entry::entry(lines, section).map(|it| ledger.entries.push(it))
    };
    read.and_then(|()| lines.exact(""))
}

/// Whether the next line is an unchecked purpose of `section`.
fn is_purpose(section: Section, line: Option<&str>) -> bool {
    let unchecked = section == Section::Unchecked;
    unchecked && line.is_some_and(|it| it.starts_with(PURPOSE_LINE))
}

/// An unchecked purpose.
fn purpose(lines: &mut Lines<'_>) -> Result<UncheckedPurpose, LedgerError> {
    lines
        .after(PURPOSE_LINE)
        .and_then(json::<WirePurpose>)
        .map(UncheckedPurpose::from)
}

#[cfg(test)]
pub(super) mod tests;
