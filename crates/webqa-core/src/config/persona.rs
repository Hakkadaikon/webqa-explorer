//! The persona of an interview run: who uses the service, and how.

use serde::Deserialize;

use super::ConfigError;
use super::interview::InterviewError;
use super::interview::document::{is_blank, read};

/// The person the judge plays during an interview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Persona {
    /// The name.
    pub name: String,
    /// Who the person is: attributes and when they use the service.
    pub profile: String,
    /// How used to computers and phones the person is.
    pub it_literacy: String,
    /// What the person wants to get done with the service.
    pub wants: String,
    /// What the person does not know or does not do. Never empty.
    pub avoids: Vec<String>,
}

/// Reads and checks a persona document.
///
/// # Errors
///
/// [`InterviewError::Malformed`] when the document is not YAML of the
/// expected shape, and [`InterviewError::MissingField`] for the first field
/// that is absent or blank.
pub fn parse(text: &str) -> Result<Persona, ConfigError> {
    let raw: RawPersona = read("persona", text)?;
    Ok(raw.check()?)
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPersona {
    name: Option<String>,
    profile: Option<String>,
    it_literacy: Option<String>,
    wants: Option<String>,
    /// A null item reads as `None` and counts as blank.
    avoids: Option<Vec<Option<String>>>,
}

impl RawPersona {
    /// The persona, or the first field that is absent or blank.
    fn check(self) -> Result<Persona, InterviewError> {
        self.first_missing().map_or_else(
            || Ok(self.into_persona()),
            |field| Err(InterviewError::MissingField(field)),
        )
    }

    /// The first field, in the order written in the documentation, that is
    /// absent or blank.
    fn first_missing(&self) -> Option<&'static str> {
        [
            ("persona.name", text_missing(self.name.as_deref())),
            ("persona.profile", text_missing(self.profile.as_deref())),
            (
                "persona.it_literacy",
                text_missing(self.it_literacy.as_deref()),
            ),
            ("persona.wants", text_missing(self.wants.as_deref())),
            ("persona.avoids", list_missing(self.avoids.as_deref())),
        ]
        .into_iter()
        .find_map(|(field, missing)| missing.then_some(field))
    }

    /// The persona. Only called once no field is missing.
    fn into_persona(self) -> Persona {
        Persona {
            name: self.name.unwrap_or_default(),
            profile: self.profile.unwrap_or_default(),
            it_literacy: self.it_literacy.unwrap_or_default(),
            wants: self.wants.unwrap_or_default(),
            avoids: self.avoids.into_iter().flatten().flatten().collect(),
        }
    }
}

/// Absent, or blank after trimming.
fn text_missing(value: Option<&str>) -> bool {
    value.is_none_or(is_blank)
}

/// Absent, empty, or with a null or blank item.
fn list_missing(items: Option<&[Option<String>]>) -> bool {
    items.is_none_or(|items| {
        items.is_empty() || items.iter().map(Option::as_deref).any(text_missing)
    })
}

#[cfg(test)]
pub(crate) mod tests;
