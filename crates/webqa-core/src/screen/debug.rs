//! `Debug` that keeps page strings out: names and values show as `…`.

use std::fmt;

use super::{Element, ElementId, Screen, ScreenId, ScreenText};

const HIDDEN: &str = "…";

impl fmt::Debug for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Element")
            .field("role", &self.role)
            .field("name", &HIDDEN)
            .field("value", &self.value.as_ref().map(|_| HIDDEN))
            .field("rect", &self.rect)
            .finish()
    }
}

impl fmt::Debug for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Screen")
            .field("path", &self.url.path())
            .field("elements", &self.elements.len())
            .finish()
    }
}

impl fmt::Debug for ScreenText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScreenText")
            .field("lines", &self.0.lines().count())
            .finish()
    }
}

impl fmt::Debug for ScreenId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScreenId")
            .field("path", &self.path)
            .field("elements", &self.elements.len())
            .finish()
    }
}

impl fmt::Debug for ElementId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElementId")
            .field("role", &self.role)
            .field("name", &HIDDEN)
            .field("nth", &self.nth)
            .finish()
    }
}
