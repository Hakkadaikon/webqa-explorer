//! Finding the element a touch names, and the value to type into it.
//!
//! A touch names its element by an id made from the redacted, normalized
//! name. The adapter holds the raw names, so it builds the same ids with the
//! same redactor.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use url::Url;
use webqa_core::config::Secret;
use webqa_core::redact::Redactor;
use webqa_core::screen::{ElementId, Screen, element_ids};

use crate::snapshot::Entry;

/// The entry whose id, built as the core builds it, equals `target`.
pub fn resolve<'a>(
    target: &ElementId,
    entries: &'a [Entry],
    url: &Url,
    redactor: &Redactor,
) -> Option<&'a Entry> {
    let screen = Screen {
        url: url.clone(),
        elements: entries.iter().map(|e| e.element.clone()).collect(),
    };
    let ids = element_ids(&redactor.screen(screen));
    ids.into_iter()
        .zip(entries)
        .find(|(id, _)| id == target)
        .map(|(_, entry)| entry)
}

/// The value of the input whose key is exactly the raw name of the element.
pub fn input_value<'a>(entry: &Entry, inputs: &'a BTreeMap<String, Secret>) -> Option<&'a str> {
    inputs.get(&entry.element.name).map(Secret::expose)
}
