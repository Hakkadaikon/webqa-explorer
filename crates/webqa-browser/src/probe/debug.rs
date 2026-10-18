//! `Debug` that keeps destinations out: a URL shows as `…`.

use std::fmt;

use super::{Probe, RawProbe};

const HIDDEN: &str = "…";

impl fmt::Debug for RawProbe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawProbe")
            .field("hit", &self.hit)
            .field("href", &self.href.as_ref().map(|_| HIDDEN))
            .field("form_action", &self.form_action.as_ref().map(|_| HIDDEN))
            .field("forbidden", &self.forbidden)
            .field("allowed", &self.allowed)
            .finish()
    }
}

impl fmt::Debug for Probe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Probe")
            .field("hit", &self.hit)
            .field("href", &self.href.as_ref().map(|_| HIDDEN))
            .field("form_action", &self.form_action.as_ref().map(|_| HIDDEN))
            .field("forbidden_listed", &self.forbidden_listed)
            .field("allowed_listed", &self.allowed_listed)
            .finish()
    }
}
