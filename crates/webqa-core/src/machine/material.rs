//! What the browser collects for the guardrails, and the forbidden elements
//! decided from it.

use std::collections::BTreeSet;

use super::Seen;
use super::rules::Rules;
use crate::guard::forbidden::Candidate;
use crate::guard::identity::Node;
use crate::guard::paywall::PaywallFacts;
use crate::screen::{Element, Screen, Viewport, visible};

/// The material of the guardrails, before redaction. It stays in the core,
/// and `Debug` shows no page strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Material {
    /// The material of the paywall check.
    pub paywall: PaywallFacts,
    /// The material of the forbidden check, in the order of `screen.elements`.
    pub candidates: Vec<Candidate>,
    /// The element references, in the order of `screen.elements`.
    pub nodes: Vec<Node>,
}

impl Material {
    /// The material known from the page alone, until the browser collects it.
    ///
    /// A paywall is found by its URL only. Nodes are referenced by position,
    /// so an element added above makes the others look replaced.
    #[must_use]
    pub fn plain(screen: &Screen, viewport: Viewport) -> Self {
        Self {
            paywall: PaywallFacts {
                url: screen.url.clone(),
                headings: Vec::new(),
                dialogs: Vec::new(),
                buttons: buttons(screen, viewport),
                hits: Vec::new(),
            },
            candidates: screen.elements.iter().map(candidate).collect(),
            nodes: screen.elements.iter().zip(0..).map(node).collect(),
        }
    }
}

/// The names of the buttons inside the viewport, in page order.
fn buttons(screen: &Screen, viewport: Viewport) -> Vec<String> {
    visible(screen, viewport)
        .into_iter()
        .filter(|e| e.role == "button")
        .map(|e| e.name.clone())
        .collect()
}

fn candidate(e: &Element) -> Candidate {
    Candidate {
        role: e.role.clone(),
        name: e.name.clone(),
        href: None,
        form_action: None,
        forbidden_listed: false,
        allowed_listed: false,
    }
}

fn node((e, position): (&Element, usize)) -> Node {
    Node {
        role: e.role.clone(),
        name: e.name.clone(),
        node: format!("p{position}"),
    }
}

/// The positions in `screen.elements` that must not be operated. A position
/// without a candidate is forbidden too.
pub(super) fn forbidden_indices(rules: &Rules, seen: &Seen) -> BTreeSet<usize> {
    let candidates = &seen.material.candidates;
    (0..seen.screen.as_inner().elements.len())
        .filter(|&i| candidates.get(i).is_none_or(|c| rules.forbidden.forbids(c)))
        .collect()
}
