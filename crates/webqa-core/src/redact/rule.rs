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

    /// Checks the rule after it was joined into `combined`.
    ///
    /// Compiling the rule alone stops a pattern such as `a)|(b` that compiles
    /// only when joined. Before this rule, `combined` did not match the empty
    /// string, so an empty match now comes from this rule.
    pub(super) fn check(&self, combined: &Regex) -> Result<(), InvalidRule> {
        self.compile(&self.regex)?;
        self.reject_empty(combined)
    }

    pub(super) fn compile(&self, source: &str) -> Result<Regex, InvalidRule> {
        Regex::new(source).map_err(|e| self.invalid(Reason::Syntax(e)))
    }

    fn reject_empty(&self, regex: &Regex) -> Result<(), InvalidRule> {
        if regex.is_match("") {
            return Err(self.invalid(Reason::MatchesEmpty));
        }
        Ok(())
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

/// A rule that cannot be used. The message names the rule, never its value or pattern.
#[derive(Debug)]
pub struct InvalidRule {
    name: String,
    reason: Reason,
}

#[derive(Debug)]
enum Reason {
    Syntax(regex::Error),
    MatchesEmpty,
}

impl Reason {
    const fn message(&self) -> &'static str {
        match self {
            Self::Syntax(_) => "is not a valid regular expression",
            Self::MatchesEmpty => "matches the empty string",
        }
    }
}

impl fmt::Display for InvalidRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "redaction rule \"{}\" {}",
            self.name,
            self.reason.message()
        )
    }
}

impl Error for InvalidRule {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.reason {
            Reason::Syntax(e) => Some(e),
            Reason::MatchesEmpty => None,
        }
    }
}
