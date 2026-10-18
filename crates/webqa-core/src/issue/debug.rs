//! `Debug` that keeps page strings out: evidence shows as `…`.

use std::fmt;

use super::Issue;

const HIDDEN: &str = "…";

impl fmt::Debug for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Issue")
            .field("kind", &self.kind)
            .field("evidence", &self.evidence.as_ref().map(|_| HIDDEN))
            .field("label", &self.label)
            .field("p", &self.p)
            .finish()
    }
}
