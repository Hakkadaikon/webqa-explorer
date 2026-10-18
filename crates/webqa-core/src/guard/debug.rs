//! `Debug` that keeps page strings out: names and URLs show as `…` or counts.

#[cfg(test)]
mod tests;

use std::fmt;

use super::forbidden::Candidate;
use super::identity::{Node, Target};
use super::paywall::PaywallFacts;

const HIDDEN: &str = "…";

impl fmt::Debug for Candidate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Candidate")
            .field("role", &self.role)
            .field("name", &HIDDEN)
            .field("href", &self.href.as_ref().map(|_| HIDDEN))
            .field("form_action", &self.form_action.as_ref().map(|_| HIDDEN))
            .field("forbidden_listed", &self.forbidden_listed)
            .field("allowed_listed", &self.allowed_listed)
            .finish()
    }
}

impl fmt::Debug for PaywallFacts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PaywallFacts")
            .field("headings", &self.headings.len())
            .field("dialogs", &self.dialogs.len())
            .field("buttons", &self.buttons.len())
            .field("hits", &self.hits.len())
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("role", &self.role)
            .field("name", &HIDDEN)
            .field("node", &self.node)
            .finish()
    }
}

impl fmt::Debug for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Target")
            .field("role", &self.role)
            .field("name", &HIDDEN)
            .field("nth", &self.nth)
            .field("node", &self.node)
            .finish()
    }
}
