//! The screen as the agent sees it: elements, viewport, identity and change.

mod debug;
#[cfg(test)]
mod identity_tests;
#[cfg(test)]
mod privacy_tests;
#[cfg(test)]
mod tests;

use std::collections::{BTreeSet, HashMap};

use crate::redact::Redacted;

/// A box in CSS px, relative to the viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// Left edge.
    pub x: i64,
    /// Top edge.
    pub y: i64,
    /// Width.
    pub width: i64,
    /// Height.
    pub height: i64,
}

/// The size of the viewport in CSS px.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    /// Width.
    pub width: i64,
    /// Height.
    pub height: i64,
}

/// An operable element. `Debug` hides the name and the value.
#[derive(Clone, PartialEq, Eq)]
pub struct Element {
    /// ARIA role.
    pub role: String,
    /// Accessible name.
    pub name: String,
    /// Current value of an input.
    pub value: Option<String>,
    /// Box on screen, if the page reported one.
    pub rect: Option<Rect>,
}

/// A page: its URL and every operable element in page order.
#[derive(Clone, PartialEq, Eq)]
pub struct Screen {
    /// URL of the page.
    pub url: url::Url,
    /// Operable elements of the whole page, in page order.
    pub elements: Vec<Element>,
}

/// The viewport's elements as numbered lines.
#[derive(Clone, PartialEq, Eq)]
pub struct ScreenText(String);

impl ScreenText {
    /// Borrows the text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Identifies a screen: the URL path and the set of (role, normalized name).
#[derive(Clone, PartialEq, Eq)]
pub struct ScreenId {
    /// Path of the URL, without the query.
    pub path: String,
    /// (role, normalized name) of every element of the page.
    pub elements: BTreeSet<(String, String)>,
}

/// Identifies an element within a screen.
#[derive(Clone, PartialEq, Eq)]
pub struct ElementId {
    /// ARIA role.
    pub role: String,
    /// Normalized name.
    pub name: String,
    /// Position among elements with the same role and normalized name, from 0.
    pub nth: usize,
}

/// Whether the element's box overlaps the viewport by at least 1px.
///
/// No box, or a box with no width or height, is outside.
#[must_use]
pub fn in_viewport(element: &Element, viewport: Viewport) -> bool {
    element.rect.is_some_and(|r| {
        overlaps(r.x, r.width, viewport.width) && overlaps(r.y, r.height, viewport.height)
    })
}

/// Whether `[start, start + len)` and `[0, limit)` share at least one px.
fn overlaps(start: i64, len: i64, limit: i64) -> bool {
    start.saturating_add(len).min(limit) > start.max(0)
}

/// Elements inside the viewport, in page order.
#[must_use]
pub fn visible(screen: &Screen, viewport: Viewport) -> Vec<&Element> {
    screen
        .elements
        .iter()
        .filter(|e| in_viewport(e, viewport))
        .collect()
}

/// Numbered lines of the elements inside the viewport, such as `1: button "保存"`.
#[must_use]
pub fn screen_text(screen: &Redacted<Screen>, viewport: Viewport) -> Redacted<ScreenText> {
    screen.map(|s| {
        let lines: Vec<String> = visible(s, viewport)
            .into_iter()
            .zip(1..)
            .map(|(e, n)| line(n, e))
            .collect();
        ScreenText(lines.join("\n"))
    })
}

fn line(n: usize, e: &Element) -> String {
    let value = e
        .value
        .as_ref()
        .map_or_else(String::new, |v| format!(" value={}", quote(v)));
    format!("{n}: {} {}{value}", e.role, quote(&e.name))
}

/// Wraps `s` in `"`, escaping `\`, `"` and newlines with a backslash.
fn quote(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

/// Whether the viewport text or the URL differs.
#[must_use]
pub fn changed(prev: &Redacted<Screen>, next: &Redacted<Screen>, viewport: Viewport) -> bool {
    prev.as_inner().url.as_str() != next.as_inner().url.as_str()
        || screen_text(prev, viewport) != screen_text(next, viewport)
}

/// Trims, collapses whitespace runs to one space and digit runs to `#`.
#[must_use]
pub fn normalize_name(name: &str) -> String {
    let mut out = String::new();
    let mut prev = None;
    for c in name.trim().chars() {
        prev = push_collapsed(&mut out, prev, c);
    }
    out
}

/// Pushes `c`, or its run character unless the run is already open. Returns the run.
fn push_collapsed(out: &mut String, prev: Option<char>, c: char) -> Option<char> {
    let class = collapsed(c);
    if class.is_none() || class != prev {
        out.push(class.unwrap_or(c));
    }
    class
}

/// The character a run of `c` collapses to, if `c` belongs to a run.
fn collapsed(c: char) -> Option<char> {
    if c.is_whitespace() {
        Some(' ')
    } else {
        is_digit(c).then_some('#')
    }
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit() || ('０'..='９').contains(&c)
}

/// The identity of the screen. The query and the values are not used.
#[must_use]
pub fn screen_id(screen: &Redacted<Screen>) -> ScreenId {
    let screen = screen.as_inner();
    ScreenId {
        path: screen.url.path().to_owned(),
        elements: screen
            .elements
            .iter()
            .map(|e| (e.role.clone(), normalize_name(&e.name)))
            .collect(),
    }
}

/// The identity of every element of the page, in page order.
#[must_use]
pub fn element_ids(screen: &Redacted<Screen>) -> Vec<ElementId> {
    let mut seen: HashMap<(String, String), usize> = HashMap::new();
    screen
        .as_inner()
        .elements
        .iter()
        .map(|e| {
            let name = normalize_name(&e.name);
            let count = seen.entry((e.role.clone(), name.clone())).or_default();
            let nth = *count;
            *count += 1;
            ElementId {
                role: e.role.clone(),
                name,
                nth,
            }
        })
        .collect()
}
