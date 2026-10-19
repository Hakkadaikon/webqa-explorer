//! Writing a ledger as Markdown: the head, then the sections.

mod entry;

use serde_json::Value;

use super::label::{
    CHECKED, EMPTY, FINISHED, GOAL, INFERRED, NO_TRIAL, PERSONA, PURPOSE_LINE, SECTIONS, START,
    TITLE, UNCHECKED_SIGHT, VERSION, WRITTEN,
};
use super::mask::masked;
use super::wire::{WirePurpose, json, quoted};
use super::{Entry, Goal, Head, Ledger};
use crate::interview::issue::{UncheckedPurpose, rank};
use crate::interview::natural::Section;
use crate::redact::Redactor;

/// The ledger as Markdown, with the strings redacted.
#[must_use]
pub fn write(ledger: &Ledger, redactor: &Redactor) -> String {
    let ledger = masked(ledger, redactor);
    let sections = SECTIONS
        .iter()
        .flat_map(|&(section, heading)| section_lines(&ledger, section, heading));
    let mut text = String::new();
    for line in head_lines(&ledger.head).into_iter().chain(sections) {
        text.push_str(&line);
        text.push('\n');
    }
    text
}

/// The lines of the head, with the blank after it.
fn head_lines(head: &Head) -> Vec<String> {
    let sight = if head.images {
        CHECKED
    } else {
        UNCHECKED_SIGHT
    };
    vec![
        TITLE.to_owned(),
        String::new(),
        VERSION.to_owned(),
        format!("{PERSONA}{}", quoted(&head.persona)),
        format!("{GOAL}{}", goal(&head.goal)),
        format!("{START}{}", quoted(&head.start)),
        format!("{FINISHED}{}", finished(head.finished)),
        sight.to_owned(),
        String::new(),
    ]
}

/// The purpose after its prefix.
fn goal(goal: &Goal) -> String {
    match goal {
        Goal::Written(s) => format!("{WRITTEN}{}", quoted(s)),
        Goal::Inferred(items) => format!("{INFERRED}{}", Value::from(items.clone())),
    }
}

/// The finished trials after their prefix.
fn finished(n: usize) -> String {
    if n == 0 {
        NO_TRIAL.to_owned()
    } else {
        n.to_string()
    }
}

/// The lines of `section`: its heading, its issues in the ledger order,
/// and the unchecked purposes in the unchecked section.
fn section_lines(ledger: &Ledger, section: Section, heading: &str) -> Vec<String> {
    let mut entries: Vec<&Entry> = ledger
        .entries
        .iter()
        .filter(|it| it.section == section)
        .collect();
    entries.sort_by(|a, b| rank(&a.issue, &b.issue));
    let purposes = ledger
        .unchecked_purposes
        .iter()
        .filter(|_| section == Section::Unchecked)
        .flat_map(purpose_lines);
    let body: Vec<String> = entries
        .into_iter()
        .flat_map(entry::lines)
        .chain(purposes)
        .collect();
    [heading.to_owned(), String::new()]
        .into_iter()
        .chain(or_empty(body))
        .collect()
}

/// `body`, or the line of an empty section with its blank.
fn or_empty(body: Vec<String>) -> Vec<String> {
    if body.is_empty() {
        vec![EMPTY.to_owned(), String::new()]
    } else {
        body
    }
}

/// The line of an unchecked purpose, with its blank.
fn purpose_lines(purpose: &UncheckedPurpose) -> [String; 2] {
    let line = format!("{PURPOSE_LINE}{}", json(&WirePurpose::from(purpose)));
    [line, String::new()]
}

#[cfg(test)]
mod tests;
