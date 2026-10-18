//! Redaction rules and why a rule cannot be used.

use std::cmp::Reverse;
use std::error::Error;
use std::fmt;

use regex::Regex;

/// A named rule: what to call the secret and how to find it.
///
/// `Debug` shows only the name, never the value or the pattern.
#[derive(Clone)]
pub struct Rule {
    pub(super) name: String,
    pub(super) regex: String,
    /// Literals come first, longest first; patterns keep the configured order.
    pub(super) rank: (bool, Reverse<usize>),
}

impl Rule {
    /// Finds the secret by a regular expression.
    #[must_use]
    pub fn pattern(name: &str, pattern: &str) -> Self {
        Self {
            name: name.to_owned(),
            regex: pattern.to_owned(),
            rank: (true, Reverse(0)),
        }
    }

    /// Finds the secret by a fixed string, such as a value read from the environment.
    #[must_use]
    pub fn literal(name: &str, value: &str) -> Self {
        Self {
            name: name.to_owned(),
            regex: regex::escape(value),
            rank: (false, Reverse(value.len())),
        }
    }

    /// Compiles the rule alone, so that a pattern such as `a)|(b`, which
    /// compiles only when joined, is rejected.
    pub(super) fn check(&self) -> Result<(), InvalidRule> {
        Regex::new(&self.regex)
            .map(drop)
            .map_err(|_| self.invalid(Reason::SYNTAX))
    }

    /// Compiles the rules joined so far, ending with this one.
    ///
    /// The `regex::Error` is dropped, as it holds the other rules' values.
    /// Before this rule, the joined rules did not match the empty string, so an
    /// empty match now comes from this rule.
    pub(super) fn join(&self, joined: &str) -> Result<Regex, InvalidRule> {
        let regex = Regex::new(joined).map_err(|_| self.invalid(Reason::INCOMPATIBLE))?;
        self.reject_empty(regex)
    }

    fn reject_empty(&self, regex: Regex) -> Result<Regex, InvalidRule> {
        if regex.is_match("") {
            return Err(self.invalid(Reason::MATCHES_EMPTY));
        }
        Ok(regex)
    }

    fn invalid(&self, reason: Reason) -> InvalidRule {
        InvalidRule {
            name: self.name.clone(),
            reason,
        }
    }
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rule")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

/// A rule that cannot be used. It holds only the rule's name and the reason,
/// never a value or a pattern.
pub struct InvalidRule {
    name: String,
    reason: Reason,
}

/// Why a rule cannot be used.
#[derive(Clone, Copy)]
struct Reason(&'static str);

impl Reason {
    const SYNTAX: Self = Self("is not a valid regular expression");
    const INCOMPATIBLE: Self = Self("cannot be combined with the other rules");
    const MATCHES_EMPTY: Self = Self("matches the empty string");
}

impl fmt::Display for InvalidRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "redaction rule \"{}\" {}", self.name, self.reason.0)
    }
}

impl fmt::Debug for InvalidRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InvalidRule")
            .field("name", &self.name)
            .field("reason", &self.reason.0)
            .finish()
    }
}

impl Error for InvalidRule {}
