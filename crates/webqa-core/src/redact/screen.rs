//! Masks the page strings of a screen.

use url::Url;

use super::{Redacted, Redactor};
use crate::screen::{Element, Screen};

impl Redactor {
    /// Masks the secrets in the URL path and query, and in the name and value
    /// of every element.
    #[must_use]
    pub fn screen(&self, mut screen: Screen) -> Redacted<Screen> {
        self.url(&mut screen.url);
        for element in &mut screen.elements {
            self.element(element);
        }
        Redacted(screen)
    }

    /// Masks each path segment, so the `/` between them stay, then the query.
    fn url(&self, url: &mut Url) {
        let segments: Vec<String> = url.path().split('/').map(|s| self.mask(s)).collect();
        url.set_path(&segments.join("/"));
        let query = url.query().map(|q| self.mask(q));
        url.set_query(query.as_deref());
    }

    fn element(&self, element: &mut Element) {
        element.name = self.mask(&element.name);
        element.value = element.value.as_deref().map(|v| self.mask(v));
    }
}
