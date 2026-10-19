//! The memory file: the JSON a memory is saved as, and reading it back.
//!
//! Screens are named `s1`, `s2`, … by their position. A screen's identity
//! is not written: it is made again from the path and the elements.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Edge, ElementOp, Entry, Memory, Node, ScreenKey};
use crate::screen::{ElementId, ScreenId};

/// The version of the format.
const SCHEMA: u64 = 1;

/// Why a memory file cannot be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(rename_all = "snake_case")]
pub enum MemoryFileError {
    /// Not JSON, or not the shape of a memory file.
    #[error("探索の記憶を読めない")]
    Unreadable,
    /// Another version of the format.
    #[error("探索の記憶の形式の版が違う")]
    Schema,
    /// The memory of another origin.
    #[error("探索の記憶が別の対象のもの")]
    Origin,
}

/// The whole file.
#[derive(Serialize, Deserialize)]
struct File {
    schema: u64,
    origin: String,
    screens: BTreeMap<String, ScreenFile>,
    edges: Vec<EdgeFile>,
}

/// One screen.
#[derive(Serialize, Deserialize)]
struct ScreenFile {
    path: String,
    elements: Vec<EntryFile>,
}

/// One element of a screen and its marks.
#[derive(Serialize, Deserialize)]
struct EntryFile {
    role: String,
    name: String,
    nth: usize,
    operated: bool,
    #[serde(default)]
    forbidden: bool,
}

/// An element, as an edge names it.
#[derive(Serialize, Deserialize)]
struct IdFile {
    role: String,
    name: String,
    nth: usize,
}

/// One edge.
#[derive(Serialize, Deserialize)]
struct EdgeFile {
    from: String,
    element: IdFile,
    op: ElementOp,
    to: String,
}

impl Memory {
    /// The memory as the JSON of its file, for `origin`.
    #[must_use]
    pub fn to_json(&self, origin: &str) -> Value {
        let file = File {
            schema: SCHEMA,
            origin: origin.to_owned(),
            screens: self.screens.iter().zip(1..).map(named).collect(),
            edges: self.edges.iter().map(EdgeFile::of).collect(),
        };
        // Every field serializes to JSON, so this never falls back.
        serde_json::to_value(file).unwrap_or_default()
    }

    /// Reads a memory file written for `origin`.
    ///
    /// # Errors
    ///
    /// [`MemoryFileError`] when the text is not a memory file of version 1
    /// for `origin`.
    pub fn from_json(text: &str, origin: &str) -> Result<Self, MemoryFileError> {
        let value: Value = serde_json::from_str(text).or(Err(MemoryFileError::Unreadable))?;
        check(&value, origin)?;
        read(value)
    }
}

/// The version first, then the origin.
fn check(value: &Value, origin: &str) -> Result<(), MemoryFileError> {
    schema(value)?;
    same_origin(value, origin)
}

/// A missing version, or one that is not a number, makes the file unreadable.
fn schema(value: &Value) -> Result<(), MemoryFileError> {
    match value.get("schema").and_then(Value::as_u64) {
        Some(SCHEMA) => Ok(()),
        Some(_) => Err(MemoryFileError::Schema),
        None => Err(MemoryFileError::Unreadable),
    }
}

/// A missing origin is left to the shape.
fn same_origin(value: &Value, origin: &str) -> Result<(), MemoryFileError> {
    let other = value
        .get("origin")
        .and_then(Value::as_str)
        .is_some_and(|written| written != origin);
    if other {
        Err(MemoryFileError::Origin)
    } else {
        Ok(())
    }
}

/// The memory, when the value has the shape of a memory file.
fn read(value: Value) -> Result<Memory, MemoryFileError> {
    serde_json::from_value::<File>(value)
        .ok()
        .and_then(File::memory)
        .ok_or(MemoryFileError::Unreadable)
}

/// The screen at `position` (from 1) with its name.
fn named((node, position): (&Node, usize)) -> (String, ScreenFile) {
    (format!("s{position}"), ScreenFile::of(node))
}

/// The number of a screen's name `s<n>`, written without a sign or zeros
/// in front.
fn number(name: &str) -> Option<usize> {
    name.strip_prefix('s')
        .and_then(|digits| digits.parse::<usize>().ok())
        .filter(|n| format!("s{n}") == name)
}

/// The key of the screen named `name` among `count` screens.
fn key(name: &str, count: usize) -> Option<ScreenKey> {
    number(name)
        .and_then(|n| n.checked_sub(1))
        .filter(|position| *position < count)
        .map(ScreenKey)
}

impl File {
    /// The memory, when the screens are numbered from 1 without gaps and
    /// every edge joins two of them.
    fn memory(self) -> Option<Memory> {
        let screens = in_order(self.screens)?;
        let count = screens.len();
        let edges = self.edges.into_iter().map(|e| e.edge(count));
        Some(Memory {
            edges: edges.collect::<Option<Vec<Edge>>>()?,
            screens: screens.into_iter().map(ScreenFile::node).collect(),
        })
    }
}

/// The screens in the order of their numbers, when they are numbered from
/// 1 without gaps.
fn in_order(screens: BTreeMap<String, ScreenFile>) -> Option<Vec<ScreenFile>> {
    let numbered = screens
        .into_iter()
        .map(numbered)
        .collect::<Option<BTreeMap<usize, ScreenFile>>>()?;
    let count = numbered.len();
    let gapless = numbered.keys().copied().eq(1..=count);
    gapless.then(|| numbered.into_values().collect())
}

/// The screen with the number of its name.
fn numbered((name, screen): (String, ScreenFile)) -> Option<(usize, ScreenFile)> {
    number(&name).map(|n| (n, screen))
}

impl ScreenFile {
    fn of(node: &Node) -> Self {
        Self {
            path: node.id.path.clone(),
            elements: node.elements.iter().map(EntryFile::of).collect(),
        }
    }

    /// The screen, its identity made from the path and the elements.
    fn node(self) -> Node {
        let elements: Vec<Entry> = self.elements.into_iter().map(EntryFile::entry).collect();
        let id = ScreenId {
            path: self.path,
            elements: elements
                .iter()
                .map(|e| (e.id.role.clone(), e.id.name.clone()))
                .collect(),
        };
        Node { id, elements }
    }
}

impl EntryFile {
    fn of(entry: &Entry) -> Self {
        Self {
            role: entry.id.role.clone(),
            name: entry.id.name.clone(),
            nth: entry.id.nth,
            operated: entry.operated,
            forbidden: entry.forbidden,
        }
    }

    fn entry(self) -> Entry {
        Entry {
            id: ElementId {
                role: self.role,
                name: self.name,
                nth: self.nth,
            },
            operated: self.operated,
            forbidden: self.forbidden,
        }
    }
}

impl IdFile {
    fn of(id: &ElementId) -> Self {
        Self {
            role: id.role.clone(),
            name: id.name.clone(),
            nth: id.nth,
        }
    }

    fn id(self) -> ElementId {
        ElementId {
            role: self.role,
            name: self.name,
            nth: self.nth,
        }
    }
}

impl EdgeFile {
    fn of(edge: &Edge) -> Self {
        Self {
            from: format!("s{}", edge.from.0.saturating_add(1)),
            element: IdFile::of(&edge.element),
            op: edge.op,
            to: format!("s{}", edge.to.0.saturating_add(1)),
        }
    }

    /// The edge, when both ends are among `count` screens.
    fn edge(self, count: usize) -> Option<Edge> {
        let ends = key(&self.from, count).zip(key(&self.to, count));
        ends.map(|(from, to)| Edge {
            from,
            element: self.element.id(),
            op: self.op,
            to,
        })
    }
}
