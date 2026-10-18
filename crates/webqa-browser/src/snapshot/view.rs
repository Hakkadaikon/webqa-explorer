//! What the snapshot shows: the screen, the paywall's words, the primary actions.

use webqa_core::screen::{Element, Rect, Screen, Viewport, in_viewport};

use super::{Entry, Snapshot};

impl Snapshot {
    /// The page as the screen model wants it.
    #[must_use]
    pub fn screen(&self, url: url::Url) -> Screen {
        Screen {
            url,
            elements: self.entries.iter().map(|e| e.element.clone()).collect(),
        }
    }

    /// Names of visible headings, then of visible elements inside dialogs.
    ///
    /// Empty names are left out.
    #[must_use]
    pub fn paywall_texts(&self, viewport: Viewport) -> Vec<String> {
        let headings = self
            .headings
            .iter()
            .filter(|h| shows(h.rect, viewport))
            .map(|h| &h.name);
        headings
            .chain(self.dialog_names(viewport))
            .filter(|name| !name.is_empty())
            .cloned()
            .collect()
    }

    /// Names of the visible elements inside dialogs, in page order.
    fn dialog_names(&self, viewport: Viewport) -> impl Iterator<Item = &String> {
        self.entries
            .iter()
            .filter(move |e| e.in_dialog && in_viewport(&e.element, viewport))
            .map(|e| &e.element.name)
    }

    /// Visible elements outside dialogs, in page order: the primary actions.
    #[must_use]
    pub fn primary(&self, viewport: Viewport) -> Vec<&Entry> {
        self.entries
            .iter()
            .filter(|e| !e.in_dialog)
            .filter(|e| in_viewport(&e.element, viewport))
            .collect()
    }
}

/// Whether a box overlaps the viewport, judged as for an element.
fn shows(rect: Option<Rect>, viewport: Viewport) -> bool {
    let probe = Element {
        role: String::new(),
        name: String::new(),
        value: None,
        rect,
    };
    in_viewport(&probe, viewport)
}
