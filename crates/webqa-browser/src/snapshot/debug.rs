//! `Debug` that keeps page strings out: names and values show as `…`.

use std::fmt;

use super::{Entry, Heading, Snapshot};

const HIDDEN: &str = "…";

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry")
            .field("element", &self.element)
            .field("node", &self.node)
            .field("in_dialog", &self.in_dialog)
            .finish()
    }
}

impl fmt::Debug for Heading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Heading")
            .field("name", &HIDDEN)
            .field("rect", &self.rect)
            .finish()
    }
}

impl fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Snapshot")
            .field("entries", &self.entries.len())
            .field("headings", &self.headings.len())
            .finish()
    }
}
