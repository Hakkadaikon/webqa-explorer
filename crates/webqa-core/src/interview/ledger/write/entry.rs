//! The lines of one issue: the first line and the fields.

use std::iter;

use serde_json::Value;

use crate::interview::issue::{Detail, Place, clue_label};
use crate::interview::ledger::label::{
    COLON, COMPARED, FIELD, FIELDS, NO_ELEMENT, PATTERN_MARK, name_of,
};
use crate::interview::ledger::wire::{
    WireAct, WireMismatch, WireShot, WireTarget, WireTrait, json, quoted,
};
use crate::interview::ledger::{Entry, instruction};

/// The values of the fields, in the order of [`FIELDS`].
const VALUES: [fn(&Entry) -> String; 14] = [
    kind,
    screen,
    mark,
    element,
    selector,
    shot,
    path,
    persona_trait,
    reason,
    expectation,
    occurred,
    reached,
    confidence,
    compared,
];

/// The lines of `entry`: the first line, the fields and a blank.
pub(super) fn lines(entry: &Entry) -> Vec<String> {
    let fields = FIELDS
        .iter()
        .zip(VALUES)
        .map(|(name, value)| format!("{FIELD}{name}{COLON}{}", value(entry)));
    iter::once(first_line(entry))
        .chain(fields)
        .chain([String::new()])
        .collect()
}

/// The checkbox, the number and the instruction.
fn first_line(entry: &Entry) -> String {
    let mark = if entry.selected { 'x' } else { ' ' };
    let issue = &entry.issue;
    format!("- [{mark}] {} {}", issue.number, instruction(issue))
}

fn kind(entry: &Entry) -> String {
    entry.issue.kind().label().to_owned()
}

/// The URL or the pattern, and the screen mark.
fn place(entry: &Entry) -> (String, String) {
    match &entry.issue.place {
        Place::Screen { url, mark } => (quoted(url), mark.to_string()),
        Place::Pattern(pattern) => (quoted(pattern), PATTERN_MARK.to_owned()),
    }
}

fn screen(entry: &Entry) -> String {
    place(entry).0
}

fn mark(entry: &Entry) -> String {
    place(entry).1
}

fn element(entry: &Entry) -> String {
    json(&entry.issue.target.as_ref().map(WireTarget::from))
}

fn selector(entry: &Entry) -> String {
    entry.issue.target.as_ref().map_or_else(
        || NO_ELEMENT.to_owned(),
        |t| {
            let (role, name) = (quoted(&t.role), quoted(&t.name));
            format!("getByRole({role}, {{ name: {name} }}).nth({})", t.nth)
        },
    )
}

fn shot(entry: &Entry) -> String {
    json(&entry.issue.shot.map(WireShot::from))
}

fn path(entry: &Entry) -> String {
    let acts: Vec<WireAct> = entry.issue.path.iter().map(WireAct::from).collect();
    json(&acts)
}

fn persona_trait(entry: &Entry) -> String {
    json(&entry.persona_trait.as_ref().map(WireTrait::from))
}

/// The reason and the expectation, by kind.
fn detail(entry: &Entry) -> (String, String) {
    match &entry.issue.detail {
        Detail::Hesitant { clues, expectation } => {
            let names: Vec<&str> = clues.iter().copied().map(clue_label).collect();
            (json(&names), quoted(expectation))
        }
        Detail::Visibility {
            reason,
            expectation,
        } => (quoted(reason), quoted(expectation)),
        Detail::Unreached { halt, expectation } => (quoted(halt.label()), json(expectation)),
        Detail::Mismatch { purpose, guesses } => {
            let wire = WireMismatch {
                purpose: purpose.clone(),
                guesses: guesses.clone(),
            };
            (json(&wire), Value::Null.to_string())
        }
    }
}

fn reason(entry: &Entry) -> String {
    detail(entry).0
}

fn expectation(entry: &Entry) -> String {
    detail(entry).1
}

fn occurred(entry: &Entry) -> String {
    let count = entry.issue.count;
    format!("{}回の試行のうち{}回", count.finished, count.occurred)
}

fn reached(entry: &Entry) -> String {
    entry.issue.count.reached.to_string()
}

fn confidence(entry: &Entry) -> String {
    Value::from(entry.issue.confidence.filter(|c| c.is_finite())).to_string()
}

fn compared(entry: &Entry) -> String {
    name_of(&COMPARED, entry.compared).to_owned()
}
