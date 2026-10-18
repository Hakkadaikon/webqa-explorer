//! Element existence and identity: the element is still there and is the
//! same node as when the judge was asked.

#[cfg(test)]
mod tests;

/// An operable element of the whole page, in page order.
///
/// `Debug` hides the name.
#[derive(Clone, PartialEq, Eq)]
pub struct Node {
    /// ARIA role.
    pub role: String,
    /// Accessible name.
    pub name: String,
    /// Element reference of the `Ai` snapshot.
    pub node: String,
}

/// The element as it was when the judge was asked.
///
/// `Debug` hides the name.
#[derive(Clone, PartialEq, Eq)]
pub struct Target {
    /// ARIA role.
    pub role: String,
    /// Accessible name, not normalized: a renamed node is another element.
    pub name: String,
    /// Position among the elements with the same role and name, from 0.
    pub nth: usize,
    /// Element reference of the `Ai` snapshot.
    pub node: String,
}

impl Target {
    /// The target for the `index`-th node (from 0), or `None` out of range.
    #[must_use]
    pub fn at(nodes: &[Node], index: usize) -> Option<Self> {
        let picked = nodes.get(index)?;
        let nth = nodes
            .iter()
            .take(index)
            .filter(|n| same_kind(n, &picked.role, &picked.name))
            .count();
        Some(Self {
            role: picked.role.clone(),
            name: picked.name.clone(),
            nth,
            node: picked.node.clone(),
        })
    }
}

/// What became of the target in the refetched list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recheck {
    /// The same node: it may be operated.
    Same,
    /// No element with that role, name and position.
    Missing,
    /// Another node in its place.
    Replaced,
}

/// Looks the target up again in the list refetched just before operating.
#[must_use]
pub fn recheck(target: &Target, nodes: &[Node]) -> Recheck {
    nodes
        .iter()
        .filter(|n| same_kind(n, &target.role, &target.name))
        .nth(target.nth)
        .map_or(Recheck::Missing, |n| same_node(target, n))
}

fn same_kind(n: &Node, role: &str, name: &str) -> bool {
    n.role == role && n.name == name
}

fn same_node(target: &Target, n: &Node) -> Recheck {
    if n.node == target.node {
        Recheck::Same
    } else {
        Recheck::Replaced
    }
}
