//! The last check that a masked part shows no secret at any depth.
//!
//! Masking a part keeps its encoding, which can leave a secret that shows
//! only once the part is decoded further. So the result is decoded again,
//! up to three times in each reading, and compared with the whole part
//! decoded and masked. When the two differ or a rule still matches, the part
//! is replaced as a whole by that decoded and masked form, encoded once.

use percent_encoding::utf8_percent_encode;

use super::super::Redactor;
use super::Part;
use super::layer::{deepest, layers};

impl Redactor {
    /// `masked` (`encoded` itself when `None`) when it shows no secret.
    /// Otherwise the whole part decoded, masked and encoded once, or the
    /// empty string when even that still shows one.
    pub(super) fn checked(
        &self,
        encoded: &str,
        masked: Option<String>,
        part: &Part,
    ) -> Option<String> {
        let reference = self.reference(encoded, part.readings);
        let candidate = masked.as_deref().unwrap_or(encoded);
        if self.hides_all(candidate, &reference, part.readings) {
            masked
        } else {
            Some(self.whole(&reference, part))
        }
    }

    /// `reference` encoded once, or the empty string when that still shows
    /// a secret.
    fn whole(&self, reference: &str, part: &Part) -> String {
        let whole = utf8_percent_encode(reference, part.kept).to_string();
        let hidden = self.hides_all(&whole, reference, part.readings);
        if hidden { whole } else { String::new() }
    }

    /// The part decoded as deep as it goes and masked, in each reading in
    /// turn.
    fn reference(&self, encoded: &str, readings: &[bool]) -> String {
        readings.iter().fold(encoded.to_owned(), |text, &plus| {
            self.mask(&deepest(&text, plus))
        })
    }

    /// Whether `candidate` decodes to `reference` and no rule matches it at
    /// any depth, in any reading.
    fn hides_all(&self, candidate: &str, reference: &str, readings: &[bool]) -> bool {
        self.reference(candidate, readings) == reference
            && !readings.iter().any(|&plus| self.shows(candidate, plus))
    }

    /// Whether a rule matches any decoding of `encoded`.
    fn shows(&self, encoded: &str, plus: bool) -> bool {
        layers(encoded, plus).any(|layer| self.masked(&layer).is_some())
    }
}
