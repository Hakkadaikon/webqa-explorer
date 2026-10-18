//! YAML's quoted scalars as `renderAriaSnapshotAsYaml` writes them.

use std::str::Chars;

/// A line, or a part of one, that does not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Bad;

/// Reads a single-quoted scalar from just after its opening `'`.
///
/// Returns the text and what follows the closing `'`. `''` is one `'`.
pub(super) fn single(s: &str) -> Result<(String, &str), Bad> {
    single_from(s, String::new())
}

fn single_from(s: &str, out: String) -> Result<(String, &str), Bad> {
    s.split_once('\'')
        .ok_or(Bad)
        .and_then(|(part, after)| single_step(part, after, out))
}

/// Adds `part`, then ends at a lone `'` or goes on past `''`.
fn single_step<'a>(part: &str, after: &'a str, mut out: String) -> Result<(String, &'a str), Bad> {
    out.push_str(part);
    match after.strip_prefix('\'') {
        Some(more) => {
            out.push('\'');
            single_from(more, out)
        }
        None => Ok((out, after)),
    }
}

/// Reads a double-quoted scalar from just after its opening `"`.
///
/// What follows the closing `"` is not read. Every turn of the loop takes
/// a character off the text, so the loop ends with it.
pub(super) fn double(s: &str) -> Result<String, Bad> {
    let mut chars = s.chars();
    let mut out = String::new();
    while let Some(c) = chars.next() {
        if c == '"' {
            return Ok(out);
        }
        out.push(content(c, &mut chars)?);
    }
    Err(Bad)
}

/// The character `c` stands for, reading the rest of an escape from `chars`.
fn content(c: char, chars: &mut Chars<'_>) -> Result<char, Bad> {
    if c == '\\' { escaped(chars) } else { Ok(c) }
}

/// Characters written with a short backslash escape, by the letter after it.
const ESCAPES: [(char, char); 7] = [
    ('\\', '\\'),
    ('"', '"'),
    ('b', '\u{8}'),
    ('f', '\u{c}'),
    ('n', '\n'),
    ('r', '\r'),
    ('t', '\t'),
];

/// The character an escape stands for, from just after its `\`.
fn escaped(chars: &mut Chars<'_>) -> Result<char, Bad> {
    let letter = chars.next().ok_or(Bad)?;
    if letter == 'x' {
        hex(chars)
    } else {
        short(letter)
    }
}

fn short(letter: char) -> Result<char, Bad> {
    ESCAPES
        .iter()
        .find(|(from, _)| *from == letter)
        .map(|(_, to)| *to)
        .ok_or(Bad)
}

/// `\xNN`: two hexadecimal digits, and nothing else (no sign).
fn hex(chars: &mut Chars<'_>) -> Result<char, Bad> {
    let digits: String = chars.take(2).collect();
    Some(digits)
        .filter(|d| d.chars().all(|c| c.is_ascii_hexdigit()))
        .and_then(|d| u32::from_str_radix(&d, 16).ok())
        .and_then(char::from_u32)
        .ok_or(Bad)
}
