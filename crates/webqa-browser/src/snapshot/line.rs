//! One element line: the key (role, name, attributes) and its value.

use std::borrow::Cow;

use super::attrs::{self, Attrs};
use super::quoted::{self, Bad};

/// An element line, read.
pub(super) struct Line {
    /// ARIA role.
    pub role: String,
    /// Accessible name; empty when the key has none.
    pub name: String,
    /// The used attributes.
    pub attrs: Attrs,
    /// The text after `: `, if any.
    pub value: Option<String>,
}

/// Reads the part of an element line after `- `.
pub(super) fn element(body: &str) -> Result<Line, Bad> {
    let (key, after) = split_key(body)?;
    let value = tail(after)?;
    let (role, name, attrs) = parts(&key)?;
    Ok(Line {
        role: role.to_owned(),
        name,
        attrs,
        value,
    })
}

/// The key, unquoted, and what follows it.
fn split_key(body: &str) -> Result<(Cow<'_, str>, &str), Bad> {
    body.strip_prefix('\'').map_or_else(
        || Ok(unquoted(body)),
        |s| quoted::single(s).map(|(key, after)| (Cow::Owned(key), after)),
    )
}

/// A bare key ends at the first `": "` or at a `:` that ends the line.
fn unquoted(body: &str) -> (Cow<'_, str>, &str) {
    let end = body
        .find(": ")
        .or_else(|| body.strip_suffix(':').map(str::len))
        .unwrap_or(body.len());
    let (key, after) = body.split_at(end);
    (Cow::Borrowed(key), after)
}

/// The value from what follows the key: nothing, `:` alone, or `: value`.
fn tail(after: &str) -> Result<Option<String>, Bad> {
    if after.is_empty() {
        return Ok(None);
    }
    after.strip_prefix(':').ok_or(Bad).and_then(scalar)
}

/// The value after a `:`, unquoted; `None` when nothing follows.
pub(super) fn scalar(text: &str) -> Result<Option<String>, Bad> {
    let text = text.strip_prefix(' ').unwrap_or(text);
    (!text.is_empty())
        .then(|| plain_or_quoted(text))
        .transpose()
}

fn plain_or_quoted(text: &str) -> Result<String, Bad> {
    text.strip_prefix('"')
        .map_or_else(|| Ok(text.to_owned()), quoted::double)
}

/// Role, name and attributes of a key.
fn parts(key: &str) -> Result<(&str, String, Attrs), Bad> {
    let (role, rest) = role(key)?;
    let (name, rest) = name(rest)?;
    attrs::read(rest).map(|attrs| (role, name, attrs))
}

/// The role: one or more ASCII lowercase letters.
fn role(key: &str) -> Result<(&str, &str), Bad> {
    let end = key
        .find(|c: char| !c.is_ascii_lowercase())
        .unwrap_or(key.len());
    let (role, rest) = key.split_at(end);
    (!role.is_empty()).then_some((role, rest)).ok_or(Bad)
}

/// The name in JSON after a space, or empty when there is none.
fn name(rest: &str) -> Result<(String, &str), Bad> {
    rest.strip_prefix(' ')
        .filter(|s| s.starts_with('"'))
        .map_or_else(|| Ok((String::new(), rest)), json)
}

/// A JSON string at the start of `text`, and what follows it.
fn json(text: &str) -> Result<(String, &str), Bad> {
    let mut stream = serde_json::Deserializer::from_str(text).into_iter::<String>();
    let name = stream.next().and_then(Result::ok).ok_or(Bad)?;
    Ok((name, text.split_at(stream.byte_offset()).1))
}
