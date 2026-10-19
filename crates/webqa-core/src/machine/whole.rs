//! The whole page that `done_when` reads: every element, operable or not,
//! and the text, both redacted.

#[cfg(test)]
mod tests;

use std::fmt;

use crate::redact::Redacted;
use crate::screen::Screen;
use crate::stage::Page;

/// The whole page that `done_when` reads, redacted.
#[derive(Clone, PartialEq, Eq)]
pub struct Whole {
    /// Every element of the page, operable or not. Its URL is the page's URL.
    pub outline: Redacted<Screen>,
    /// The text of the whole page.
    pub text: Redacted<String>,
}

impl Whole {
    /// What the page alone tells, until the browser collects the whole page:
    /// the operable elements, and their names joined with "\n" as the text.
    #[must_use]
    pub fn plain(screen: &Redacted<Screen>) -> Self {
        Self {
            outline: screen.clone(),
            text: screen.map(names),
        }
    }

    /// The material of `stage::goal`.
    #[must_use]
    pub fn page(&self) -> Page<'_> {
        let outline = self.outline.as_inner();
        Page {
            url: &outline.url,
            elements: &outline.elements,
            text: self.text.as_inner(),
        }
    }
}

/// The names of the elements in page order, one per line.
fn names(screen: &Screen) -> String {
    let names: Vec<&str> = screen.elements.iter().map(|e| e.name.as_str()).collect();
    names.join("\n")
}

/// Shows the outline, which hides the names, and leaves out the text.
impl fmt::Debug for Whole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Whole")
            .field("outline", &self.outline)
            .finish_non_exhaustive()
    }
}
