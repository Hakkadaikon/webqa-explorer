//! Masks the secrets in a URL, matched on its decoded form.
//!
//! A part is first read with its `%` escapes decoded once, and each match is
//! written over the span of the encoded string behind it. Then each piece of
//! the part is decoded up to three times, and a piece whose deeper decoding
//! shows a secret is replaced as a whole. Last, the result is checked for a
//! secret at any depth, and the whole part is replaced when one still shows.
//! What masking does not touch keeps its encoding, so a URL with no secret
//! stays as it was.

mod check;
#[cfg(test)]
mod embedded_tests;
mod layer;
mod pieces;
#[cfg(test)]
mod tests;

use std::ops::Range;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use url::Url;

use super::{Redacted, Redactor};
use layer::Decoded;

/// What a written name or a replaced piece leaves unescaped: the unreserved
/// marks and the brackets of `[[name]]`.
const WRITTEN: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~')
    .remove(b'[')
    .remove(b']');

/// How one part of a URL is read and written.
struct Part {
    /// Whether a `+` reads as a space, in each reading in turn.
    readings: &'static [bool],
    /// What splits the part into pieces.
    separators: &'static [char],
    /// What the part replaced as a whole leaves unescaped.
    kept: &'static AsciiSet,
}

/// A `+` is itself, and each segment is a piece.
const PATH: Part = Part {
    readings: &[false],
    separators: &['/'],
    kept: &WRITTEN.remove(b'/'),
};

/// A `+` reads as itself first, then as a space. Each name and each value is
/// a piece.
const QUERY: Part = Part {
    readings: &[false, true],
    separators: &['&', '='],
    kept: &WRITTEN.remove(b'&').remove(b'='),
};

/// A `+` is itself, and the whole fragment is one piece.
const FRAGMENT: Part = Part {
    readings: &[false],
    separators: &[],
    kept: WRITTEN,
};

impl Redactor {
    /// The URL as it may leave the process: its path, query and fragment
    /// masked as in [`Redactor::screen`], without the user name and password,
    /// written with `Url::as_str`.
    #[must_use]
    pub fn url(&self, url: &Url) -> Redacted<String> {
        let mut url = url.clone();
        self.mask_url(&mut url);
        // Both fail only for a URL that cannot hold a user name.
        let _ = url.set_password(None);
        let _ = url.set_username("");
        Redacted(url.as_str().to_owned())
    }

    /// Masks the path, the query and the fragment.
    pub(super) fn mask_url(&self, url: &mut Url) {
        self.mask_path(url);
        self.mask_query(url);
        self.mask_fragment(url);
    }

    fn mask_path(&self, url: &mut Url) {
        if let Some(path) = self.mask_part(url.path(), &PATH) {
            url.set_path(&path);
        }
    }

    fn mask_query(&self, url: &mut Url) {
        let query = url.query().and_then(|q| self.mask_part(q, &QUERY));
        if query.is_some() {
            url.set_query(query.as_deref());
        }
    }

    fn mask_fragment(&self, url: &mut Url) {
        let fragment = url.fragment().and_then(|f| self.mask_part(f, &FRAGMENT));
        if fragment.is_some() {
            url.set_fragment(fragment.as_deref());
        }
    }

    /// One encoded part, masked in each of its readings and checked for a
    /// secret left at any depth. `None` when nothing was masked.
    fn mask_part(&self, encoded: &str, part: &Part) -> Option<String> {
        let readings = part.readings;
        let spans = each_reading(encoded, readings, |s, plus| self.mask_spans(s, plus));
        let current = spans.as_deref().unwrap_or(encoded);
        let pieces = each_reading(current, readings, |s, plus| {
            self.mask_pieces(s, part.separators, plus)
        })
        .or(spans);
        self.checked(encoded, pieces, part)
    }

    /// `encoded` decoded once, with each match written over the span of
    /// `encoded` behind it. `None` when nothing matched.
    fn mask_spans(&self, encoded: &str, plus: bool) -> Option<String> {
        let decoded = Decoded::new(encoded, plus);
        let mut out = String::with_capacity(encoded.len());
        let mut at = 0;
        for (span, name) in self.spans(&decoded) {
            out.push_str(encoded.get(at..span.start).unwrap_or_default());
            out.extend(utf8_percent_encode(name, WRITTEN));
            at = span.end;
        }
        out.push_str(encoded.get(at..).unwrap_or_default());
        (at > 0).then_some(out)
    }

    /// Each match in the decoded text, as the encoded span behind it and the
    /// name written in its place.
    fn spans<'a>(
        &'a self,
        decoded: &'a Decoded,
    ) -> impl Iterator<Item = (Range<usize>, &'a str)> + 'a {
        self.regex
            .iter()
            .flat_map(|regex| regex.captures_iter(&decoded.text))
            .filter_map(|caps| {
                let name = self.name_of(&caps);
                decoded
                    .span(&caps.get_match().range())
                    .map(|span| (span, name))
            })
    }

    /// The masked string, only when masking changed it.
    fn masked(&self, decoded: &str) -> Option<String> {
        let masked = self.mask(decoded);
        (masked != decoded).then_some(masked)
    }
}

/// Applies `mask` in each reading, each to what the one before left.
/// `None` when no reading masked anything.
fn each_reading(
    encoded: &str,
    readings: &[bool],
    mask: impl Fn(&str, bool) -> Option<String>,
) -> Option<String> {
    readings.iter().fold(None, |done, &plus| {
        mask(done.as_deref().unwrap_or(encoded), plus).or(done)
    })
}
