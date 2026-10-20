//! The redaction rules a run applies, built from the configuration.

use webqa_core::config::{Config, Redact, RedactSource, Secret};
use webqa_core::redact::Rule;

/// The name written in place of the API key.
const API_KEY: &str = "APIキー";

/// The redaction rules of the configuration: `redact` in order, then each
/// input value under its key, unless it is `secret: false`, then the API key
/// as `APIキー`. Empty values make no rule.
#[must_use]
pub fn rules(config: &Config) -> Vec<Rule> {
    let redact = config.redact.iter().filter_map(configured);
    let inputs = config
        .inputs
        .iter()
        .filter(|(key, _)| !config.open_inputs.contains(*key))
        .filter_map(|(key, value)| literal(key, value));
    let key = config
        .judge
        .api_key
        .iter()
        .filter_map(|value| literal(API_KEY, value));
    redact.chain(inputs).chain(key).collect()
}

/// The rule of one `redact` entry.
fn configured(entry: &Redact) -> Option<Rule> {
    match &entry.source {
        RedactSource::Pattern(pattern) => Some(Rule::pattern(&entry.name, pattern)),
        RedactSource::Env(value) => literal(&entry.name, value),
    }
}

/// A rule for the value itself. An empty value would match everywhere, so it
/// makes none.
fn literal(name: &str, value: &Secret) -> Option<Rule> {
    let value = value.expose();
    (!value.is_empty()).then(|| Rule::literal(name, value))
}

#[cfg(test)]
mod tests {
    use webqa_core::event::Mode;
    use webqa_core::redact::Redactor;

    use super::super::tests::{ALL_ENV, EMPTY_EMAIL, EXAMPLE, changed, config, example};
    use super::*;

    /// The names of the rules, as their `Debug` shows them.
    fn names(rules: &[Rule]) -> Vec<String> {
        rules.iter().map(|rule| format!("{rule:?}")).collect()
    }

    fn rule(name: &str) -> String {
        format!("Rule {{ name: {name:?}, .. }}")
    }

    #[test]
    fn redaction_rules_cover_inputs_and_key() {
        let setup = example(Mode::Explore);
        let text = "mail value-of-TEST_EMAIL key value-of-JEV_API_KEY a@b.cd";
        let want = "mail [[メールアドレス]] key [[APIキー]] [[メールアドレス]]";
        assert_eq!(setup.redactor.text(text).into_inner(), want, "redacted");
        let all = rules(&config(EXAMPLE, ALL_ENV));
        let email = rule("メールアドレス");
        let key = rule("APIキー");
        let three = [email.clone(), email.clone(), key.clone()];
        assert_eq!(names(&all), three, "redact, inputs, then the key");
        let empty = rules(&config(EXAMPLE, EMPTY_EMAIL));
        assert_eq!(names(&empty), [email, key], "no empty input");
        assert_eq!(Redactor::new(&empty).ok().map(drop), Some(()), "usable");
    }

    #[test]
    fn inputs_marked_not_secret_make_no_rule() {
        let yaml = changed("{ env: TEST_EMAIL }", "{ env: TEST_EMAIL, secret: false }");
        let all = rules(&config(&yaml, ALL_ENV));
        assert_eq!(
            names(&all),
            [rule("メールアドレス"), rule("APIキー")],
            "the pattern and the key only"
        );
    }

    #[test]
    fn values_from_the_environment_come_first_unless_empty() {
        let yaml = changed("redact:\n", "redact:\n  - { name: トークン, env: TOKEN }\n");
        let email = rule("メールアドレス");
        let key = rule("APIキー");
        let four = [rule("トークン"), email.clone(), email.clone(), key];
        let token = rules(&config(&yaml, ALL_ENV));
        assert_eq!(names(&token), four, "the env value first");
        let blank = rules(&config(&yaml, &|_| Some(String::new())));
        assert_eq!(names(&blank), [email], "only the pattern");
    }

    #[test]
    fn redactor_keeps_not_secret_values_and_hides_the_rest() {
        let yaml = changed(
            "  メールアドレス: { env: TEST_EMAIL }",
            "  検索語: { env: TEST_QUERY, secret: false }\n  \
             会員番号: { env: TEST_ID }",
        );
        let all = rules(&config(&yaml, ALL_ENV));
        let redactor = Redactor::new(&all).unwrap();
        let text = "q value-of-TEST_QUERY id value-of-TEST_ID";
        let want = "q value-of-TEST_QUERY id [[会員番号]]";
        assert_eq!(redactor.text(text).into_inner(), want, "redacted");
    }
}
