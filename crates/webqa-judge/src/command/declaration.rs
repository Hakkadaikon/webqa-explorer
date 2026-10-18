//! The version line the bridge writes when it starts, before it reads any
//! request: `{"version":2,"images":true}`. Unknown fields are ignored.

use serde::Deserialize;
use serde::de::DeserializeOwned;

use super::PROTOCOL_VERSION;
use crate::table::StartError;

/// The version, read before anything else.
#[derive(Deserialize)]
struct Head {
    version: u32,
}

/// What the bridge declared.
#[derive(Deserialize)]
struct Declaration {
    images: bool,
}

/// Reads the version line the bridge writes when it starts: whether it
/// reads images.
///
/// The version is checked first: [`StartError::Version`] when it is not
/// [`PROTOCOL_VERSION`]. A line that is not a JSON object with a numeric
/// version and a boolean `images` is [`StartError::Declaration`].
pub fn read(line: &[u8]) -> Result<bool, StartError> {
    let head = parse::<Head>(line)?;
    if head.version != PROTOCOL_VERSION {
        return Err(StartError::Version);
    }
    parse::<Declaration>(line).map(|declaration| declaration.images)
}

fn parse<T: DeserializeOwned>(line: &[u8]) -> Result<T, StartError> {
    serde_json::from_slice(line)
        .ok()
        .ok_or(StartError::Declaration)
}

#[cfg(test)]
mod tests;
