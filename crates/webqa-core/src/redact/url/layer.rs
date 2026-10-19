//! One layer of `%` decoding that keeps where each byte came from.

use std::ops::Range;

/// The character written for bytes that are not UTF-8.
const REPLACEMENT: char = char::REPLACEMENT_CHARACTER;

/// A string decoded once, with the span of the encoded string behind each
/// byte of it.
pub struct Decoded {
    /// The decoded string. Bytes that are not UTF-8 read as U+FFFD.
    pub text: String,
    /// The span of the encoded string behind each byte of `text`.
    spans: Vec<Range<usize>>,
}

impl Decoded {
    /// Decodes each `%XX` once. With `plus`, a `+` reads as a space.
    pub fn new(encoded: &str, plus: bool) -> Self {
        let units = units(encoded.as_bytes(), plus);
        let bytes: Vec<u8> = units.iter().map(|(byte, _)| *byte).collect();
        let mut rest = units.into_iter().map(|(_, span)| span);
        let mut spans = Vec::with_capacity(bytes.len());
        for chunk in bytes.utf8_chunks() {
            spans.extend(rest.by_ref().take(chunk.valid().len()));
            let bad: Vec<Range<usize>> = rest.by_ref().take(chunk.invalid().len()).collect();
            spans.extend(replaced(&bad));
        }
        Self {
            text: String::from_utf8_lossy(&bytes).into_owned(),
            spans,
        }
    }

    /// The span of the encoded string behind `range` of `text`.
    pub fn span(&self, range: &Range<usize>) -> Option<Range<usize>> {
        let last = range.end.checked_sub(1).and_then(|i| self.spans.get(i));
        self.spans
            .get(range.start)
            .zip(last)
            .map(|(first, last)| first.start..last.end)
    }
}

/// The spans behind each byte of the U+FFFD that stands for `bad`.
fn replaced(bad: &[Range<usize>]) -> Vec<Range<usize>> {
    bad.first()
        .zip(bad.last())
        .map(|(first, last)| vec![first.start..last.end; REPLACEMENT.len_utf8()])
        .unwrap_or_default()
}

/// Each decoded byte with the span of `encoded` it came from.
fn units(encoded: &[u8], plus: bool) -> Vec<(u8, Range<usize>)> {
    let mut units = Vec::with_capacity(encoded.len());
    let mut at = 0;
    while let Some((byte, width)) = unit(encoded, at, plus) {
        units.push((byte, at..at + width));
        at += width;
    }
    units
}

/// The byte decoded at `at` and how many bytes of `encoded` it took. A `%`
/// not followed by two hex digits stays itself.
fn unit(encoded: &[u8], at: usize, plus: bool) -> Option<(u8, usize)> {
    let first = *encoded.get(at)?;
    Some(escape(encoded, at, first).map_or_else(|| (literal(first, plus), 1), |byte| (byte, 3)))
}

/// The byte of the `%XX` at `at`, if there is one.
fn escape(encoded: &[u8], at: usize, first: u8) -> Option<u8> {
    (first == b'%')
        .then(|| hex_at(encoded, at + 1).zip(hex_at(encoded, at + 2)))
        .flatten()
        .map(|(high, low)| (high << 4) | low)
}

/// A byte that is not an escape. With `plus`, a `+` reads as a space.
const fn literal(byte: u8, plus: bool) -> u8 {
    if plus && byte == b'+' { b' ' } else { byte }
}

/// The value of the hex digit at `at`.
fn hex_at(encoded: &[u8], at: usize) -> Option<u8> {
    encoded
        .get(at)
        .and_then(|byte| char::from(*byte).to_digit(16))
        .and_then(|digit| u8::try_from(digit).ok())
}
