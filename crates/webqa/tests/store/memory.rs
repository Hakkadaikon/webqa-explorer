//! The memory the tests of the store write: two screens and a click.

use std::collections::BTreeSet;

use url::{ParseError, Url};
use webqa_core::explore::{ElementOp, Memory};
use webqa_core::redact::{Redacted, Redactor};
use webqa_core::screen::{Element, ElementId, Screen};

use crate::APP;

fn element(role: &str, name: &str) -> Element {
    Element {
        role: role.to_owned(),
        name: name.to_owned(),
        value: None,
        rect: None,
    }
}

fn page(path: &str, elements: Vec<Element>) -> Result<Redacted<Screen>, ParseError> {
    let url = Url::parse(APP)?.join(path)?;
    Ok(Redactor::default().screen(Screen { url, elements }))
}

/// `/` with "保存" and "ヘルプ".
pub fn top() -> Result<Redacted<Screen>, ParseError> {
    let elements = vec![element("button", "保存"), element("link", "ヘルプ")];
    page("/", elements)
}

/// The button "保存" of `/`.
pub fn saved() -> ElementId {
    ElementId {
        role: "button".to_owned(),
        name: "保存".to_owned(),
        nth: 0,
    }
}

/// `/` and `/settings` with "通知", and the click on "保存" that moved
/// between them.
pub fn remembered() -> Result<Memory, ParseError> {
    let mut memory = Memory::default();
    let settings = page("/settings", vec![element("button", "通知")])?;
    let from = memory.visit(&top()?, &BTreeSet::new());
    let to = memory.visit(&settings, &BTreeSet::new());
    memory.record(from, &saved(), ElementOp::Click, to);
    Ok(memory)
}

/// Whether "保存" of `/` is operated, and the number of edges.
pub fn marks(memory: &mut Memory) -> Result<(bool, usize), ParseError> {
    let at = memory.visit(&top()?, &BTreeSet::new());
    Ok((memory.is_operated(at, &saved()), memory.edge_count()))
}
