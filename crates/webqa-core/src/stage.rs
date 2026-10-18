//! Stages of a fixed scenario and their `done_when`.

#[cfg(test)]
mod tests;

use crate::config::raw::{DoneWhen, Element as Wanted, Stage};
use crate::plan::Goal;
use crate::screen::Element;

/// What `done_when` reads from the page. Each covers the whole page, not only the viewport.
#[derive(Clone, Copy)]
pub struct Page<'a> {
    /// The current URL.
    pub url: &'a url::Url,
    /// Every element of the page, including ones that cannot be operated. Boxes are ignored.
    pub elements: &'a [Element],
    /// The text of the whole page.
    pub text: &'a str,
}

/// Whether every declaration written in `done_when` holds. No declaration never holds.
#[must_use]
pub fn met(done_when: &DoneWhen, page: Page<'_>) -> bool {
    let checks = [
        url_check(done_when, page),
        element_check(done_when, page),
        text_check(done_when, page),
    ];
    let written: Vec<bool> = checks.into_iter().flatten().collect();
    !written.is_empty() && !written.contains(&false)
}

/// Whether the url declaration holds, if written.
fn url_check(done_when: &DoneWhen, page: Page<'_>) -> Option<bool> {
    done_when.url.as_deref().map(|u| url_holds(u, page.url))
}

/// Whether the element declaration holds, if written.
fn element_check(done_when: &DoneWhen, page: Page<'_>) -> Option<bool> {
    done_when
        .element
        .as_ref()
        .map(|e| element_holds(e, page.elements))
}

/// Whether the text declaration holds, if written.
fn text_check(done_when: &DoneWhen, page: Page<'_>) -> Option<bool> {
    done_when.text.as_deref().map(|t| text_holds(t, page.text))
}

/// The goal of the stage at `current`. Later stages are not looked at.
///
/// No stage at `current` is unmet. A met stage advances, or achieves the goal when it
/// is the last.
#[must_use]
pub fn goal(stages: &[Stage], current: usize, page: Page<'_>) -> Goal {
    stages
        .get(current)
        .filter(|stage| met(&stage.done_when, page))
        .map_or(Goal::Unmet, |_| after(stages, current))
}

/// The goal when the stage at `current` is met.
fn after(stages: &[Stage], current: usize) -> Goal {
    stages
        .get(current + 1)
        .map_or(Goal::Achieved, |_| Goal::Advance)
}

/// Whether the text is empty or only whitespace.
fn blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// Whether a declared URL is an absolute path without a host.
fn absolute_path(declared: &str) -> bool {
    declared.starts_with('/') && !declared.starts_with("//")
}

/// Whether the path of `declared` resolved against `current` is the path of `current`.
fn same_path(declared: &str, current: &url::Url) -> bool {
    current
        .join(declared)
        .is_ok_and(|joined| joined.path() == current.path())
}

/// Whether the url declaration holds. Query and fragment are not compared.
fn url_holds(declared: &str, current: &url::Url) -> bool {
    absolute_path(declared) && same_path(declared, current)
}

/// Whether both the role and the name are written.
fn named(wanted: &Wanted) -> bool {
    !blank(&wanted.role) && !blank(&wanted.name)
}

/// Whether the element has exactly the wanted role and name.
fn same_element(element: &Element, wanted: &Wanted) -> bool {
    element.role == wanted.role && element.name == wanted.name
}

/// Whether the element declaration holds on some element of the page.
fn element_holds(wanted: &Wanted, elements: &[Element]) -> bool {
    named(wanted) && elements.iter().any(|e| same_element(e, wanted))
}

/// Whether the text declaration holds: written, and part of the page text.
fn text_holds(declared: &str, text: &str) -> bool {
    !blank(declared) && text.contains(declared)
}
