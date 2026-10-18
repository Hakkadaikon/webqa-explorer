//! Masks configured secrets before text leaves the process.

mod rule;
#[cfg(test)]
mod tests;

use std::fmt;

use regex::{Captures, Regex};
use serde_json::{Map, Value};

pub use rule::{InvalidRule, Rule};

/// A value that has passed through [`Redactor`]. Only this module can build one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redacted<T>(T);

impl<T> Redacted<T> {
    /// Borrows the redacted value.
    pub const fn as_inner(&self) -> &T {
        &self.0
    }

    /// Takes the redacted value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// Replaces every match of the rules with `[[<name>]]` in a single pass.
///
/// `Debug` shows only the rule names.
#[derive(Clone, Default)]
pub struct Redactor {
    /// One capture group per rule, joined with `|`.
    parts: Vec<String>,
    regex: Option<Regex>,
    /// The capture group of each rule and its replacement.
    groups: Vec<(usize, String)>,
}

impl Redactor {
    /// Compiles the rules into one regular expression.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidRule`] for the first rule that is not a valid regular
    /// expression or that matches the empty string.
    pub fn new(rules: &[Rule]) -> Result<Self, InvalidRule> {
        let mut ordered: Vec<&Rule> = rules.iter().collect();
        ordered.sort_by_key(|rule| rule.rank);
        ordered.into_iter().try_fold(Self::default(), Self::push)
    }

    fn push(mut self, rule: &Rule) -> Result<Self, InvalidRule> {
        let group = self.regex.as_ref().map_or(1, Regex::captures_len);
        self.parts.push(format!("({})", rule.regex));
        let regex = rule.compile(&self.parts.join("|"))?;
        rule.check(&regex)?;
        self.regex = Some(regex);
        self.groups.push((group, format!("[[{}]]", rule.name)));
        Ok(self)
    }

    /// Masks the secrets in a string.
    #[must_use]
    pub fn text(&self, s: &str) -> Redacted<String> {
        Redacted(self.mask(s))
    }

    /// Masks the secrets in every string inside a JSON value. Object keys are kept.
    #[must_use]
    pub fn json(&self, mut v: Value) -> Redacted<Value> {
        self.walk(&mut v);
        Redacted(v)
    }

    fn mask(&self, s: &str) -> String {
        self.regex
            .iter()
            .fold(s.to_owned(), |acc, regex| self.replace(regex, &acc))
    }

    fn replace(&self, regex: &Regex, s: &str) -> String {
        regex
            .replace_all(s, |caps: &Captures<'_>| self.name_of(caps))
            .into_owned()
    }

    fn name_of(&self, caps: &Captures<'_>) -> &str {
        self.groups
            .iter()
            .find(|(group, _)| caps.get(*group).is_some())
            .map_or("", |(_, name)| name.as_str())
    }

    fn walk(&self, v: &mut Value) {
        if let Value::String(s) = v {
            *s = self.mask(s);
        }
        self.walk_array(v);
        self.walk_object(v);
    }

    fn walk_array(&self, v: &mut Value) {
        for child in v.as_array_mut().into_iter().flatten() {
            self.walk(child);
        }
    }

    fn walk_object(&self, v: &mut Value) {
        for child in v.as_object_mut().into_iter().flat_map(Map::values_mut) {
            self.walk(child);
        }
    }
}

impl fmt::Debug for Redactor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self.groups.iter().map(|(_, name)| name.as_str()).collect();
        f.debug_struct("Redactor")
            .field("rules", &names)
            .finish_non_exhaustive()
    }
}
