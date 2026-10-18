//! Reading the accessibility snapshot into operable elements and their refs.

mod attrs;
mod debug;
#[cfg(test)]
mod error_tests;
mod line;
#[cfg(test)]
mod name_tests;
mod quoted;
#[cfg(test)]
mod tests;
mod tree;
#[cfg(test)]
mod tree_tests;
#[cfg(test)]
mod value_tests;
mod view;
#[cfg(test)]
mod view_tests;

use webqa_core::screen::{Element, Rect};

/// One operable element of the snapshot, before redaction.
#[derive(Clone, PartialEq, Eq)]
pub struct Entry {
    /// role, name, value and box, as the screen model wants them.
    pub element: Element,
    /// The aria ref, such as `e3` or `f1e3`.
    pub node: String,
    /// Whether a `dialog` or `alertdialog` is among its ancestors.
    pub in_dialog: bool,
}

/// A heading, kept for the paywall's words.
#[derive(Clone, PartialEq, Eq)]
pub struct Heading {
    /// Accessible name.
    pub name: String,
    /// Box on screen, if the page reported one.
    pub rect: Option<Rect>,
}

/// What the snapshot of a page holds.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct Snapshot {
    /// Operable elements of the whole page, in page order.
    pub entries: Vec<Entry>,
    /// Headings of the whole page, in page order.
    pub headings: Vec<Heading>,
}

/// A line of the snapshot that does not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("アクセシビリティ情報の {line} 行目を読めない")]
pub struct SnapshotError {
    /// 1 から数えた行番号。
    pub line: usize,
}

/// Reads the text of `aria_snapshot` in the `Ai` mode with boxes.
///
/// # Errors
///
/// [`SnapshotError`] with the number of the first line that does not read.
pub fn parse(text: &str) -> Result<Snapshot, SnapshotError> {
    let mut tree = tree::Tree::default();
    for (line, number) in text.lines().zip(1..) {
        tree.push(line)
            .map_err(|_| SnapshotError { line: number })?;
    }
    Ok(tree.finish())
}
