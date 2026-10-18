//! Tests of the exploration memory, and builders shared with the candidate tests.

mod finished;
mod memory;
mod reach;
mod visit;

use std::collections::BTreeSet;

use super::{ElementOp, Memory, ScreenKey};
use crate::explore::select::Candidate;
use crate::redact::{Redacted, Redactor};
use crate::screen::{Element, ElementId, Rect, Screen, Viewport};

pub const VIEW: Viewport = Viewport {
    width: 800,
    height: 600,
};

pub const NONE: BTreeSet<usize> = BTreeSet::new();

/// An element whose 10x10 box starts at `(0, y)`.
pub fn at(role: &str, name: &str, y: i64) -> Element {
    Element {
        role: role.to_owned(),
        name: name.to_owned(),
        value: None,
        rect: Some(Rect {
            x: 0,
            y,
            width: 10,
            height: 10,
        }),
    }
}

/// An element inside the viewport.
pub fn shown(role: &str, name: &str) -> Element {
    at(role, name, 0)
}

/// An element below the viewport.
pub fn below(role: &str, name: &str) -> Element {
    at(role, name, 700)
}

/// A screen of `https://app.example.test` passed through a redactor with no rules.
pub fn page(path: &str, elements: Vec<Element>) -> Redacted<Screen> {
    let url = format!("https://app.example.test{path}");
    Redactor::default().screen(Screen {
        url: url::Url::parse(&url).unwrap(),
        elements,
    })
}

/// A screen of buttons inside the viewport.
pub fn buttons(path: &str, names: &[&str]) -> Redacted<Screen> {
    page(path, names.iter().map(|n| shown("button", n)).collect())
}

pub fn id(role: &str, name: &str, nth: usize) -> ElementId {
    ElementId {
        role: role.to_owned(),
        name: name.to_owned(),
        nth,
    }
}

pub fn button(name: &str) -> ElementId {
    id("button", name, 0)
}

pub fn forbid(numbers: &[usize]) -> BTreeSet<usize> {
    numbers.iter().copied().collect()
}

pub fn candidate(number: usize, role: &str, name: &str, nth: usize) -> Candidate {
    Candidate {
        number,
        id: id(role, name, nth),
    }
}

/// Clicks `element` on `from`, moving to `to`.
pub fn click(memory: &mut Memory, from: ScreenKey, element: &ElementId, to: ScreenKey) -> bool {
    memory.record(from, element, ElementOp::Click, to)
}
