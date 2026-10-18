//! Tests of the redaction rules.

use std::error::Error as _;

use serde_json::json;

use super::*;

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

    rejects_unusable_rules();
    replaces_literals_once_longest_first();
}

fn rejects_unusable_rules() {
    let broken = Redactor::new(&[Rule::pattern("壊れた", "(")]).unwrap_err();
    assert!(
        broken.to_string().contains("壊れた"),
        "an invalid regex is rejected with the rule's name: {broken}"
    );
    assert!(
        broken.source().is_some(),
        "the regex error is kept as the source"
    );

    let escaping = Redactor::new(&[Rule::pattern("抜け出す", "a)|(b")]).unwrap_err();
    assert!(
        escaping.to_string().contains("抜け出す"),
        "a pattern that only compiles when joined is rejected: {escaping}"
    );

    let empty_literal = Redactor::new(&[Rule::literal("空の値", "")]).unwrap_err();
    assert!(
        empty_literal.to_string().contains("空の値"),
        "an empty literal is rejected with the rule's name: {empty_literal}"
    );
    let empty_match = Redactor::new(&[Rule::pattern("空に一致", "a*")]).unwrap_err();
    assert!(
        empty_match.to_string().contains("空に一致"),
        "a pattern matching the empty string is rejected: {empty_match}"
    );
    assert!(
        empty_match.source().is_none(),
        "an empty match has no underlying regex error"
    );
    assert!(
        broken.to_string().contains("regular expression")
            && empty_match.to_string().contains("empty string"),
        "the message tells why the rule is rejected: {broken} / {empty_match}"
    );
}

fn replaces_literals_once_longest_first() {
    let lengths =
        Redactor::new(&[Rule::literal("短", "abc"), Rule::literal("長", "abcdef")]).unwrap();
    assert_eq!(
        lengths.text("abcdef").into_inner(),
        "[[長]]",
        "the longer literal wins regardless of the configured order"
    );

    let self_named = Redactor::new(&[Rule::literal("メール", "メール")]).unwrap();
    assert_eq!(
        self_named.text("メール").into_inner(),
        "[[メール]]",
        "a replacement is not replaced again"
    );

    let dotted = Redactor::new(&[Rule::literal("名前", "a.b")]).unwrap();
    assert_eq!(
        dotted.text("axb a.b").into_inner(),
        "axb [[名前]]",
        "a literal is not read as a regex"
    );
}

#[test]
fn debug_hides_values() {
    let rules = [
        Rule::literal("パスワード", "hunter2"),
        Rule::pattern("メールアドレス", EMAIL),
    ];
    let redactor = Redactor::new(&rules).unwrap();
    for shown in [
        format!("{:?}", rules[0]),
        format!("{:?}", rules[1]),
        format!("{redactor:?}"),
    ] {
        assert!(!shown.contains("hunter2"), "value leaked: {shown}");
        assert!(!shown.contains(EMAIL), "pattern leaked: {shown}");
    }
    assert!(
        format!("{:?}", rules[0]).contains("パスワード"),
        "the rule shows its name"
    );
    assert!(
        format!("{:?}", rules[1]).contains("メールアドレス"),
        "the rule shows its name"
    );
    let shown = format!("{redactor:?}");
    assert!(
        shown.contains("パスワード") && shown.contains("メールアドレス"),
        "the redactor shows the rule names: {shown}"
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
    assert_eq!(
        out["notes"][0], "見た [[メールアドレス]]",
        "strings inside arrays are masked in place"
    );
    let keyed = email().json(json!({"taro@example.test": "x"})).into_inner();
    assert!(
        keyed.get("taro@example.test").is_some(),
        "object keys are not masked: {keyed}"
    );
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
