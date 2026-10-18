//! Paywalls: screens that ask for money before going on.

#[cfg(test)]
mod tests;

use url::Url;

use super::{Segments, Words};

/// Words in a heading or inside a modal that urge to pay.
///
/// "料金" and "プラン" are left out so that pages like "料金の履歴" pass.
pub const DEFAULT_WORDS: &[&str] = &[
    "アップグレード",
    "プレミアム",
    "有料プラン",
    "有料会員",
    "upgrade",
    "premium",
    "subscribe",
];

/// Path segments of a paywall's URL.
pub const DEFAULT_SEGMENTS: &[&str] = &["pricing", "upgrade", "checkout", "subscribe", "premium"];

/// What the adapter knows of the screen. Lists are in any order.
///
/// `Debug` shows only how many entries each list has.
#[derive(Clone, PartialEq, Eq)]
pub struct PaywallFacts {
    /// URL of the page.
    pub url: Url,
    /// Headings in the viewport, outside modals.
    pub headings: Vec<String>,
    /// Names inside modals (`dialog`): their headings and buttons.
    pub dialogs: Vec<String>,
    /// Button names in the viewport, outside modals. Kept for reading, not
    /// searched for words: an "Upgrade" button in a header is no paywall.
    pub buttons: Vec<String>,
    /// For each main operation (operable, in the viewport, outside modals),
    /// whether a hit test at its center reached it.
    pub hits: Vec<bool>,
}

/// Whether more than half of the main operations are covered. False when
/// there are none.
#[must_use]
pub fn blocked(hits: &[bool]) -> bool {
    let misses = hits.iter().filter(|hit| !**hit).count();
    misses * 2 > hits.len()
}

/// Decides whether a screen is a paywall.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paywall {
    words: Words,
    segments: Segments,
}

type Axis = fn(&Paywall, &PaywallFacts) -> bool;

/// Any of these makes the screen a paywall.
const AXES: [Axis; 3] = [by_words, by_url, by_hits];

impl Paywall {
    /// The default words and segments plus the extra ones.
    #[must_use]
    pub fn new(extra_words: &[String], extra_segments: &[String]) -> Self {
        Self {
            words: Words::new(DEFAULT_WORDS, extra_words),
            segments: Segments::new(DEFAULT_SEGMENTS, extra_segments),
        }
    }

    /// Whether the screen is a paywall.
    #[must_use]
    pub fn detects(&self, f: &PaywallFacts) -> bool {
        AXES.iter().any(|axis| axis(self, f))
    }
}

impl Default for Paywall {
    fn default() -> Self {
        Self::new(&[], &[])
    }
}

/// Headings and names inside modals only.
fn by_words(p: &Paywall, f: &PaywallFacts) -> bool {
    f.headings
        .iter()
        .chain(&f.dialogs)
        .any(|text| p.words.any_in(text))
}

fn by_url(p: &Paywall, f: &PaywallFacts) -> bool {
    p.segments.any_in(&f.url)
}

fn by_hits(_: &Paywall, f: &PaywallFacts) -> bool {
    blocked(&f.hits)
}
