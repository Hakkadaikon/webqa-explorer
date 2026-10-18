//! Guardrails: pure checks that stop an operation whatever the judge says.
//!
//! Each check takes facts the browser adapter collected and knows nothing of
//! the browser or of the judge.

pub mod confidence;
mod debug;
pub mod domain;
pub mod forbidden;
pub mod identity;
mod kana;
mod path;
pub mod paywall;
#[cfg(test)]
mod tests;

use url::Url;

/// Offset from a full-width ASCII character to its half-width one.
const FULLWIDTH_OFFSET: u32 = 0xFEE0;

/// Folds text for matching: full-width ASCII to half-width, half-width
/// katakana to full-width, lower case, and runs of white space and `-` (the
/// full-width ones too) to one space, trimmed.
pub(crate) fn fold(s: &str) -> String {
    let mut wide = String::new();
    for c in s.chars().map(halfwidth).map(kana::widen) {
        kana::push(&mut wide, c);
    }
    wide.to_lowercase()
        .replace('-', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The half-width form of a full-width ASCII character (U+FF01 to U+FF5E).
/// The full-width space is left to `split_whitespace`.
fn halfwidth(c: char) -> char {
    let code = u32::from(c);
    if ('\u{FF01}'..='\u{FF5E}').contains(&c) {
        char::from_u32(code - FULLWIDTH_OFFSET).unwrap_or(c)
    } else {
        c
    }
}

/// Defaults and extra entries, each passed through `norm`, dropping the
/// empty ones: an empty entry would match everything.
fn entries(defaults: &[&str], extra: &[String], norm: fn(&str) -> String) -> Vec<String> {
    defaults
        .iter()
        .copied()
        .chain(extra.iter().map(String::as_str))
        .map(norm)
        .filter(|e| !e.is_empty())
        .collect()
}

/// Folded words. An end of a word that is an ASCII letter or digit must not
/// be continued by one in the text; other ends match anywhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Words(Vec<String>);

impl Words {
    /// The defaults and the extra words, folded.
    pub(crate) fn new(defaults: &[&str], extra: &[String]) -> Self {
        Self(entries(defaults, extra, fold))
    }

    /// Whether any word appears in the folded `text` between boundaries.
    pub(crate) fn any_in(&self, text: &str) -> bool {
        let text = fold(text);
        self.0.iter().any(|word| bounded(&text, word))
    }
}

/// Whether `word` appears in `text` without its ASCII ends being continued.
fn bounded(text: &str, word: &str) -> bool {
    text.match_indices(word).any(|(start, _)| {
        free_before(text, word, start) && free_after(text, word, start + word.len())
    })
}

/// Only a word starting with an ASCII letter or digit needs a boundary before.
fn free_before(text: &str, word: &str, at: usize) -> bool {
    !ascii_alnum(word.chars().next())
        || !ascii_alnum(text.get(..at).and_then(|s| s.chars().next_back()))
}

/// Only a word ending with an ASCII letter or digit needs a boundary after.
fn free_after(text: &str, word: &str, at: usize) -> bool {
    !ascii_alnum(word.chars().next_back())
        || !ascii_alnum(text.get(at..).and_then(|s| s.chars().next()))
}

fn ascii_alnum(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_ascii_alphanumeric())
}

/// URL path segments, compared lower case and without `-` and `_`. Segments
/// of a URL are percent-decoded and lose `;` parameters and the extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Segments(Vec<String>);

impl Segments {
    /// The defaults and the extra segments, squashed.
    pub(crate) fn new(defaults: &[&str], extra: &[String]) -> Self {
        Self(entries(defaults, extra, squash))
    }

    /// Whether a segment of the path equals one of these. The query, the
    /// fragment and the host are not looked at.
    pub(crate) fn any_in(&self, url: &Url) -> bool {
        url.path_segments()
            .is_some_and(|mut parts| parts.any(|part| self.0.contains(&path::segment_key(part))))
    }
}

/// Lower case, without `-` and `_`.
fn squash(s: &str) -> String {
    s.chars()
        .filter(|c| !['-', '_'].contains(c))
        .collect::<String>()
        .to_lowercase()
}
