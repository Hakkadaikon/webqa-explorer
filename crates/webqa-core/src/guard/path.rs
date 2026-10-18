//! The comparable form of a URL path segment.

/// Percent-decoded, lower case, without `;` parameters, the last `.`
/// extension, `-` and `_`.
pub(super) fn segment_key(segment: &str) -> String {
    let decoded = percent_decode(segment).to_lowercase();
    let bare = decoded.split(';').next().unwrap_or_default();
    let stem = bare.rsplit_once('.').map_or(bare, |(stem, _)| stem);
    super::squash(stem)
}

/// Decodes `%XX`, leaving a `%` without two hex digits as it is.
fn percent_decode(s: &str) -> String {
    let mut parts = s.split('%');
    let head = parts.next().unwrap_or_default().bytes();
    let bytes: Vec<u8> = head.chain(parts.flat_map(decode_part)).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// A part that followed a `%`.
fn decode_part(part: &str) -> Vec<u8> {
    let rest = part.get(2..).unwrap_or_default();
    hex_byte(part).map_or_else(
        || [&b"%"[..], part.as_bytes()].concat(),
        |byte| [&[byte][..], rest.as_bytes()].concat(),
    )
}

/// The byte written by the first two characters, when both are hex digits.
fn hex_byte(part: &str) -> Option<u8> {
    part.get(..2)
        .filter(|hex| hex.bytes().all(|b| b.is_ascii_hexdigit()))
        .and_then(|hex| u8::from_str_radix(hex, 16).ok())
}
