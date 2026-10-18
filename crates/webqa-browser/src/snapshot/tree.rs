//! Walks the lines in order, keeping the ancestors of the current line.

use webqa_core::screen::Element;

use super::line::{self, Line};
use super::quoted::Bad;
use super::{Entry, Heading, Snapshot};

/// Roles of operable elements.
const OPERABLE: [&str; 17] = [
    "button",
    "link",
    "textbox",
    "searchbox",
    "checkbox",
    "radio",
    "switch",
    "combobox",
    "listbox",
    "option",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "tab",
    "treeitem",
    "slider",
    "spinbutton",
];

/// Roles whose value is the text they hold.
const TYPED: [&str; 4] = ["textbox", "searchbox", "spinbutton", "combobox"];

/// Roles whose value is whether they are checked.
const CHECKED: [&str; 5] = [
    "checkbox",
    "radio",
    "switch",
    "menuitemcheckbox",
    "menuitemradio",
];

/// Roles of dialogs.
const DIALOGS: [&str; 2] = ["dialog", "alertdialog"];

type ReadValue = fn(&Line) -> Option<String>;

/// How each kind of value is read from its line.
const VALUES: [(&[&str], ReadValue); 2] = [(&TYPED, typed_value), (&CHECKED, checked_value)];

/// The snapshot read so far, and where the reader is in the tree.
#[derive(Default)]
pub(super) struct Tree {
    snapshot: Snapshot,
    /// (depth, role) of the open elements, shallowest first.
    ancestors: Vec<(usize, String)>,
    /// The depth of the text line that gives the last entry its value.
    awaiting: Option<usize>,
}

impl Tree {
    /// Reads one line.
    pub(super) fn push(&mut self, line: &str) -> Result<(), Bad> {
        if line.trim().is_empty() {
            return Ok(());
        }
        let (depth, item) = indent(line)?;
        self.item(depth, item)
    }

    pub(super) fn finish(self) -> Snapshot {
        self.snapshot
    }

    fn item(&mut self, depth: usize, item: &str) -> Result<(), Bad> {
        match item.strip_prefix("text:") {
            Some(rest) => self.text(depth, rest),
            None => self.element_or_property(depth, item),
        }
    }

    /// `- /url: …` and the like belong to the element above; skipped.
    fn element_or_property(&mut self, depth: usize, item: &str) -> Result<(), Bad> {
        if item.starts_with('/') {
            Ok(())
        } else {
            self.element(depth, item)
        }
    }

    /// `- text: …` gives its value to an input just above it.
    fn text(&mut self, depth: usize, rest: &str) -> Result<(), Bad> {
        let value = line::scalar(rest)?;
        let awaited = self.awaiting.take() == Some(depth);
        if let Some(entry) = self.snapshot.entries.last_mut().filter(|_| awaited) {
            entry.element.value = value;
        }
        Ok(())
    }

    fn element(&mut self, depth: usize, item: &str) -> Result<(), Bad> {
        let line = line::element(item)?;
        self.awaiting = None;
        let keep = self.ancestors.partition_point(|(d, _)| *d < depth);
        self.ancestors.truncate(keep);
        let in_frame = self.within(&["iframe"]);
        let in_dialog = self.within(&DIALOGS);
        self.ancestors.push((depth, line.role.clone()));
        if !in_frame {
            self.add(depth, line, in_dialog);
        }
        Ok(())
    }

    /// Whether one of the ancestors has one of `roles`.
    fn within(&self, roles: &[&str]) -> bool {
        self.ancestors
            .iter()
            .any(|(_, role)| roles.contains(&role.as_str()))
    }

    fn add(&mut self, depth: usize, line: Line, in_dialog: bool) {
        if line.role == "heading" {
            self.snapshot.headings.push(Heading {
                name: line.name,
                rect: line.attrs.rect,
            });
            return;
        }
        let entry = entry(line, in_dialog);
        self.awaiting = entry.as_ref().filter(|e| awaits_text(e)).map(|_| depth + 1);
        self.snapshot.entries.extend(entry);
    }
}

/// The depth and the part after `- ` of a line indented by two spaces a level.
fn indent(line: &str) -> Result<(usize, &str), Bad> {
    let item = line.trim_start_matches(' ');
    let spaces = line.len() - item.len();
    spaces
        .is_multiple_of(2)
        .then_some(spaces / 2)
        .zip(item.strip_prefix("- "))
        .ok_or(Bad)
}

/// The entry of an operable, enabled element with a ref.
fn entry(line: Line, in_dialog: bool) -> Option<Entry> {
    let value = value(&line);
    let wanted = operable(&line);
    line.attrs.node.filter(|_| wanted).map(|node| Entry {
        element: Element {
            role: line.role,
            name: line.name,
            value,
            rect: line.attrs.rect,
        },
        node,
        in_dialog,
    })
}

fn operable(line: &Line) -> bool {
    OPERABLE.contains(&line.role.as_str()) && !line.attrs.disabled
}

fn value(line: &Line) -> Option<String> {
    VALUES
        .iter()
        .find(|(roles, _)| roles.contains(&line.role.as_str()))
        .and_then(|(_, read)| read(line))
}

fn typed_value(line: &Line) -> Option<String> {
    line.value.clone()
}

fn checked_value(line: &Line) -> Option<String> {
    line.attrs.checked.map(str::to_owned)
}

/// Whether a text line below may still give the entry its value.
fn awaits_text(entry: &Entry) -> bool {
    TYPED.contains(&entry.element.role.as_str()) && entry.element.value.is_none()
}
