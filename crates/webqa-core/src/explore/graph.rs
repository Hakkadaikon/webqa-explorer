//! The exploration memory: screens, their elements, and the edges between them.

mod file;
#[cfg(test)]
pub(super) mod tests;

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::redact::Redacted;
use crate::screen::{ElementId, Screen, ScreenId, element_ids, screen_id};

pub use file::MemoryFileError;

/// A screen in the memory. Only [`Memory`] makes one.
///
/// It is the position in the screen list; JSON names it `s{position + 1}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenKey(usize);

/// The operation on an element that an edge records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementOp {
    /// A click.
    Click,
    /// Text input.
    Input,
}

/// The exploration memory of one target origin.
///
/// Nodes are screens, edges are "operating this element moved to that screen".
/// `Debug` hides names, since [`ScreenId`] and [`ElementId`] hide them.
#[derive(Debug, Clone, Default)]
pub struct Memory {
    screens: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Debug, Clone)]
struct Node {
    id: ScreenId,
    elements: Vec<Entry>,
}

/// An element of a screen in the memory.
#[derive(Debug, Clone)]
pub(super) struct Entry {
    pub(super) id: ElementId,
    pub(super) operated: bool,
    pub(super) forbidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Edge {
    from: ScreenKey,
    element: ElementId,
    op: ElementOp,
    to: ScreenKey,
}

impl Entry {
    /// Not operated yet and not forbidden.
    pub(super) const fn untouched(&self) -> bool {
        !self.operated && !self.forbidden
    }
}

impl Node {
    /// A new screen with every element untouched.
    fn new(id: ScreenId, ids: Vec<ElementId>, forbidden: &BTreeSet<usize>) -> Self {
        let mut node = Self {
            id,
            elements: Vec::new(),
        };
        node.merge(ids, forbidden);
        node
    }

    /// Sets the forbidden marks of the page's elements, adding unknown ones.
    ///
    /// `forbidden` holds positions in `ids`; others are ignored.
    fn merge(&mut self, ids: Vec<ElementId>, forbidden: &BTreeSet<usize>) {
        for (position, id) in ids.into_iter().enumerate() {
            self.mark(id, forbidden.contains(&position));
        }
    }

    fn mark(&mut self, id: ElementId, forbidden: bool) {
        match self.find_mut(&id) {
            Some(entry) => entry.forbidden = forbidden,
            None => self.elements.push(Entry {
                id,
                operated: false,
                forbidden,
            }),
        }
    }

    fn find(&self, id: &ElementId) -> Option<&Entry> {
        self.elements.iter().find(|e| e.id == *id)
    }

    fn find_mut(&mut self, id: &ElementId) -> Option<&mut Entry> {
        self.elements.iter_mut().find(|e| e.id == *id)
    }

    fn has_untouched(&self) -> bool {
        self.elements.iter().any(Entry::untouched)
    }
}

impl Edge {
    /// Leaves the screen `at` by operating `element`.
    fn branches_from(&self, at: ScreenKey, element: &ElementId) -> bool {
        self.from == at && self.to != at && self.element == *element
    }
}

impl Memory {
    /// Finds or adds the screen and returns its key.
    ///
    /// `forbidden` holds positions in `screen.elements`; out-of-range ones are ignored.
    /// Elements not on the page keep their marks.
    pub fn visit(&mut self, screen: &Redacted<Screen>, forbidden: &BTreeSet<usize>) -> ScreenKey {
        let id = screen_id(screen);
        let ids = element_ids(screen);
        match self.screen_mut(&id) {
            Some((node, position)) => {
                node.merge(ids, forbidden);
                ScreenKey(position)
            }
            None => self.push(Node::new(id, ids, forbidden)),
        }
    }

    /// Marks the element operated and adds the edge.
    ///
    /// True only when the element turns from untouched to operated.
    /// Unknown screens or elements change nothing.
    pub fn record(
        &mut self,
        from: ScreenKey,
        element: &ElementId,
        op: ElementOp,
        to: ScreenKey,
    ) -> bool {
        if self.screens.get(to.0).is_none() {
            return false;
        }
        let Some(entry) = self.entry_mut(from, element) else {
            return false;
        };
        let first = !entry.operated;
        entry.operated = true;
        self.add_edge(Edge {
            from,
            element: element.clone(),
            op,
            to,
        });
        first
    }

    /// Whether the element of the screen is operated.
    #[must_use]
    pub fn is_operated(&self, at: ScreenKey, element: &ElementId) -> bool {
        self.entry(at, element).is_some_and(|e| e.operated)
    }

    /// Whether the element is operated and no screen it leads to reaches an untouched element.
    ///
    /// Edges back to `at` are not counted.
    #[must_use]
    pub fn finished(&self, at: ScreenKey, element: &ElementId) -> bool {
        self.is_operated(at, element) && !self.open_branch(at, element)
    }

    /// Whether `from` or a screen reachable from it has an untouched, unforbidden element.
    #[must_use]
    pub fn untouched_reachable(&self, from: ScreenKey) -> bool {
        self.any_untouched(self.reachable(from, BTreeSet::new()))
    }

    /// The number of screens.
    #[must_use]
    pub const fn screen_count(&self) -> usize {
        self.screens.len()
    }

    /// The number of edges.
    #[must_use]
    pub const fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub(super) fn entry(&self, at: ScreenKey, element: &ElementId) -> Option<&Entry> {
        self.screens.get(at.0)?.find(element)
    }

    fn entry_mut(&mut self, at: ScreenKey, element: &ElementId) -> Option<&mut Entry> {
        self.screens.get_mut(at.0)?.find_mut(element)
    }

    /// The screen with the identity `id`, and its position.
    fn screen_mut(&mut self, id: &ScreenId) -> Option<(&mut Node, usize)> {
        self.screens.iter_mut().zip(0..).find(|(n, _)| n.id == *id)
    }

    fn push(&mut self, node: Node) -> ScreenKey {
        self.screens.push(node);
        ScreenKey(self.screens.len() - 1)
    }

    fn add_edge(&mut self, edge: Edge) {
        if !self.edges.contains(&edge) {
            self.edges.push(edge);
        }
    }

    /// Whether operating `element` on `at` leads elsewhere to an untouched element.
    fn open_branch(&self, at: ScreenKey, element: &ElementId) -> bool {
        self.edges
            .iter()
            .filter(|e| e.branches_from(at, element))
            .any(|e| self.untouched_beyond(e.to, at))
    }

    /// Whether `to` reaches an untouched element without passing through `at`.
    ///
    /// Untouched elements of `at` are left to the first tier or to scrolling.
    fn untouched_beyond(&self, to: ScreenKey, at: ScreenKey) -> bool {
        let mut seen = self.reachable(to, BTreeSet::from([at.0]));
        seen.remove(&at.0);
        self.any_untouched(seen)
    }

    /// Whether a screen among `keys` has an untouched element.
    fn any_untouched(&self, keys: BTreeSet<usize>) -> bool {
        keys.into_iter()
            .any(|key| self.screens.get(key).is_some_and(Node::has_untouched))
    }

    /// `seen` plus `from` and every screen reachable from it without entering `seen`.
    fn reachable(&self, from: ScreenKey, mut seen: BTreeSet<usize>) -> BTreeSet<usize> {
        let mut queue = vec![from];
        while let Some(key) = queue.pop() {
            self.step(key, &mut seen, &mut queue);
        }
        seen
    }

    /// Queues the targets of `key` the first time `key` is seen.
    fn step(&self, key: ScreenKey, seen: &mut BTreeSet<usize>, queue: &mut Vec<ScreenKey>) {
        if seen.insert(key.0) {
            queue.extend(self.targets(key));
        }
    }

    /// The screens that edges from `key` lead to.
    fn targets(&self, key: ScreenKey) -> impl Iterator<Item = ScreenKey> + '_ {
        self.edges
            .iter()
            .filter(move |e| e.from == key)
            .map(|e| e.to)
    }
}
