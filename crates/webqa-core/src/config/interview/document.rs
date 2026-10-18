//! Reading the documents an interview configuration points to.

use serde::de::DeserializeOwned;

use super::InterviewError;

/// Reads `text` as the document named `document`. An empty document, or one
/// with only comments, reads as the default value.
pub fn read<T: DeserializeOwned + Default>(
    document: &'static str,
    text: &str,
) -> Result<T, InterviewError> {
    serde_saphyr::from_str_with_options::<Option<T>>(text, options())
        .map(Option::unwrap_or_default)
        .map_err(|source| InterviewError::Malformed {
            document,
            source: Box::new(source),
        })
}

fn options() -> serde_saphyr::Options {
    serde_saphyr::options! { with_snippet: false }
}

/// Empty after trimming.
pub fn is_blank(text: &str) -> bool {
    text.trim().is_empty()
}
