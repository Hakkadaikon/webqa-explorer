//! Issue numbers that stay the same across runs.

use std::fmt;

use super::Kind;
use crate::screen::{ElementId, ScreenId};

/// The number of an issue, the same across runs for the same key.
/// Written as `I-` and 8 upper-case hex digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Number(u32);

/// The mark of a screen's identity, written as `S-` and 8 upper-case hex
/// digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenMark(u32);

/// What an issue's number is made from.
#[derive(Debug, Clone, Copy)]
pub struct Key<'a> {
    /// Where.
    pub spot: Spot<'a>,
    /// The kind.
    pub kind: Kind,
}

/// The place part of a key.
#[derive(Debug, Clone, Copy)]
pub enum Spot<'a> {
    /// A screen, with the element if the issue has one.
    Screen {
        /// The screen's identity.
        screen: &'a ScreenId,
        /// The element's identity, for issues with an element.
        element: Option<&'a ElementId>,
    },
    /// The URL pattern of a screen purpose, as written.
    Pattern(&'a str),
}

/// The names of the kinds in the keys.
const KIND_NAMES: [(Kind, &str); 4] = [
    (Kind::Hesitant, "hesitant"),
    (Kind::Visibility, "visibility"),
    (Kind::Unreached, "unreached"),
    (Kind::Mismatch, "mismatch"),
];

/// The FNV-1a 64-bit offset basis.
const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// The FNV-1a 64-bit prime.
const PRIME: u64 = 0x0000_0100_0000_01b3;

impl Number {
    /// The number of `key`.
    #[must_use]
    pub fn of(key: Key<'_>) -> Self {
        let mut parts = spot_parts(key.spot);
        parts.push(kind_name(key.kind).to_owned());
        Self(hash(&parts))
    }

    /// Reads `I-` and exactly 8 upper-case hex digits. `None` otherwise.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        hex8(text, "I-").map(Self)
    }
}

impl ScreenMark {
    /// The mark of `screen`.
    #[must_use]
    pub fn of(screen: &ScreenId) -> Self {
        Self(hash(&screen_parts(screen)))
    }

    /// Reads `S-` and exactly 8 upper-case hex digits. `None` otherwise.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        hex8(text, "S-").map(Self)
    }
}

/// The strings of the place part of a key.
fn spot_parts(spot: Spot<'_>) -> Vec<String> {
    match spot {
        Spot::Screen { screen, element } => {
            let mut parts = screen_parts(screen);
            parts.extend(element_parts(element));
            parts
        }
        Spot::Pattern(pattern) => vec!["pattern".to_owned(), pattern.to_owned()],
    }
}

/// `screen`, the path, the number of elements, then each role and name.
fn screen_parts(screen: &ScreenId) -> Vec<String> {
    let head = [
        "screen".to_owned(),
        screen.path.clone(),
        screen.elements.len().to_string(),
    ];
    let pairs = screen
        .elements
        .iter()
        .flat_map(|(role, name)| [role.clone(), name.clone()]);
    head.into_iter().chain(pairs).collect()
}

/// `element`, the role, the name and the position, or `none`.
fn element_parts(element: Option<&ElementId>) -> Vec<String> {
    element.map_or_else(
        || vec!["none".to_owned()],
        |e| {
            vec![
                "element".to_owned(),
                e.role.clone(),
                e.name.clone(),
                e.nth.to_string(),
            ]
        },
    )
}

/// The name of `kind` in the keys.
fn kind_name(kind: Kind) -> &'static str {
    KIND_NAMES
        .iter()
        .find(|(row, _)| *row == kind)
        .map_or("", |(_, name)| name)
}

/// The upper 32 bits of the FNV-1a 64-bit hash of the parts, each
/// written as a netstring.
fn hash(parts: &[String]) -> u32 {
    let hash = parts
        .iter()
        .flat_map(|part| netstring(part))
        .fold(OFFSET, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(PRIME)
        });
    let [b0, b1, b2, b3, _, _, _, _] = hash.to_be_bytes();
    u32::from_be_bytes([b0, b1, b2, b3])
}

/// `part` as its byte count, `:`, its bytes and `,`.
fn netstring(part: &str) -> Vec<u8> {
    format!("{}:{part},", part.len()).into_bytes()
}

/// The value written as `prefix` and exactly 8 upper-case hex digits.
fn hex8(text: &str, prefix: &str) -> Option<u32> {
    let digits = text.strip_prefix(prefix)?;
    let written = digits.len() == 8 && digits.bytes().all(is_upper_hex);
    written
        .then(|| u32::from_str_radix(digits, 16).ok())
        .flatten()
}

fn is_upper_hex(b: u8) -> bool {
    b.is_ascii_digit() || (b'A'..=b'F').contains(&b)
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "I-{:08X}", self.0)
    }
}

impl fmt::Display for ScreenMark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "S-{:08X}", self.0)
    }
}

#[cfg(test)]
mod tests;
