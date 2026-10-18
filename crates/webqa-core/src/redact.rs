//! Masks configured secrets before text leaves the process.

use regex::{NoExpand, Regex};
use serde_json::{Map, Value};

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

/// A named rule: what to call the secret and how to find it.
#[derive(Debug, Clone)]
pub struct Rule {
    name: String,
    pattern: String,
}

impl Rule {
    /// Finds the secret by a regular expression.
    #[must_use]
    pub fn pattern(name: &str, pattern: &str) -> Self {
        Self {
            name: name.to_owned(),
            pattern: pattern.to_owned(),
        }
    }

    /// Finds the secret by a fixed string, such as a value read from the environment.
    #[must_use]
    pub fn literal(name: &str, value: &str) -> Self {
        Self::pattern(name, &regex::escape(value))
    }

    fn compile(&self) -> Result<(Regex, String), regex::Error> {
        let regex = Regex::new(&self.pattern)?;
        Ok((regex, format!("[[{}]]", self.name)))
    }
}

/// Replaces every match of the rules with `[[<name>]]`.
#[derive(Debug, Clone)]
pub struct Redactor {
    rules: Vec<(Regex, String)>,
}

impl Redactor {
    /// Compiles the rules.
    ///
    /// # Errors
    ///
    /// Returns the [`regex::Error`] of the first rule whose regular expression is invalid.
    pub fn new(rules: &[Rule]) -> Result<Self, regex::Error> {
        let rules = rules.iter().map(Rule::compile).collect::<Result<_, _>>()?;
        Ok(Self { rules })
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
        self.rules.iter().fold(s.to_owned(), |acc, (regex, name)| {
            regex.replace_all(&acc, NoExpand(name)).into_owned()
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const EMAIL: &str = r"[\w.+-]+@[\w-]+\.[\w.]+";

    fn email() -> Redactor {
        Redactor::new(&[Rule::pattern("メールアドレス", EMAIL)]).unwrap()
    }

    #[test]
    fn redacts_configured_values() {
        let r = email();
        assert_eq!(
            r.text("連絡先 taro@example.test まで").as_inner(),
            "連絡先 [[メールアドレス]] まで",
            "a regex rule replaces the match with its name"
        );
        assert_eq!(
            r.text("a@b.test と a@b.test").into_inner(),
            "[[メールアドレス]] と [[メールアドレス]]",
            "every occurrence is replaced"
        );

        let password = Redactor::new(&[Rule::literal("パスワード", "hunter2")]).unwrap();
        assert_eq!(
            password.text("pw=hunter2").into_inner(),
            "pw=[[パスワード]]",
            "a literal rule replaces the fixed string"
        );

        let both = Redactor::new(&[
            Rule::pattern("メールアドレス", EMAIL),
            Rule::literal("パスワード", "hunter2"),
        ])
        .unwrap();
        assert_eq!(
            both.text("taro@example.test hunter2").into_inner(),
            "[[メールアドレス]] [[パスワード]]",
            "all rules apply"
        );

        assert_eq!(
            both.text("何もない").into_inner(),
            "何もない",
            "text matching no rule is unchanged"
        );

        assert!(
            Redactor::new(&[Rule::pattern("壊れた", "(")]).is_err(),
            "an invalid regex is rejected"
        );
    }

    #[test]
    fn redacts_names_in_event_payloads() {
        let out = email()
            .json(json!({
                "kind": "decided",
                "target": "taro@example.test",
                "notes": ["見た taro@example.test"],
                "n": 3,
                "nested": {"x": "taro@example.test"}
            }))
            .into_inner();
        let s = serde_json::to_string(&out).unwrap();
        assert!(!s.contains("taro@example.test"), "secret leaked: {s}");
        assert_eq!(
            out["target"], "[[メールアドレス]]",
            "string value is masked"
        );
        assert_eq!(out["n"], 3, "numbers are kept");
        let mut keys: Vec<&str> = out
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            ["kind", "n", "nested", "notes", "target"],
            "keys are kept"
        );
    }
}
