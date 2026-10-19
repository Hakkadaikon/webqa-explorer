//! Masks the page strings of a screen.

use super::{Redacted, Redactor};
use crate::screen::{Element, Screen};

impl Redactor {
    /// Masks the secrets in the URL path, query and fragment, matched after
    /// decoding `%` escapes, and in the name and value of every element.
    #[must_use]
    pub fn screen(&self, mut screen: Screen) -> Redacted<Screen> {
        self.mask_url(&mut screen.url);
        for element in &mut screen.elements {
            self.element(element);
        }
        Redacted(screen)
    }

    fn element(&self, element: &mut Element) {
        element.name = self.mask(&element.name);
        element.value = element.value.as_deref().map(|v| self.mask(v));
    }
}
