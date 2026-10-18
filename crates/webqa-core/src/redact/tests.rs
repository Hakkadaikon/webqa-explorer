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
    replaces_once_in_rule_order();
}

fn rejects_unusable_rules() {
    for (rules, name, why) in [
        (
            vec![Rule::pattern("壊れた", "(")],
            "壊れた",
            "regular expression",
        ),
        (
            vec![Rule::pattern("抜け出す", "a)|(b")],
            "抜け出す",
            "regular expression",
        ),
        (
            vec![
                Rule::pattern("先", "(?P<x>s3cr3t)"),
                Rule::pattern("後", "(?P<x>b)"),
            ],
            "後",
            "other rules",
        ),
        (vec![Rule::literal("空の値", "")], "空の値", "empty string"),
        (
            vec![Rule::pattern("空に一致", "a*")],
            "空に一致",
            "empty string",
        ),
    ] {
        let err = Redactor::new(&rules).unwrap_err();
        let shown = err.to_string();
        assert!(
            shown.contains(name) && shown.contains(why),
            "the error names the rule and the reason: {shown}"
        );
        assert!(err.source().is_none(), "no regex error is kept: {shown}");
    }
}

fn replaces_once_in_rule_order() {
    for (rules, input, expected, why) in [
        (
            vec![Rule::literal("短", "abc"), Rule::literal("長", "abcdef")],
            "abcdef",
            "[[長]]",
            "the longer literal wins regardless of the configured order",
        ),
        (
            vec![Rule::literal("メール", "x"), Rule::literal("B", "メール")],
            "x",
            "[[メール]]",
            "a replacement is not replaced again",
        ),
        (
            vec![Rule::pattern("p", "hunter"), Rule::literal("pw", "hunter2")],
            "hunter2",
            "[[pw]]",
            "a literal wins over a pattern at the same position",
        ),
        (
            vec![Rule::pattern("先", "ab"), Rule::pattern("後", "abc")],
            "abc",
            "[[先]]c",
            "patterns keep the configured order",
        ),
        (
            vec![Rule::literal("名前", "a.b")],
            "axb a.b",
            "axb [[名前]]",
            "a literal is not read as a regex",
        ),
    ] {
        let redactor = Redactor::new(&rules).unwrap();
        assert_eq!(redactor.text(input).into_inner(), expected, "{why}");
    }
}

fn errors_hide_other_rules() {
    for (rules, secret) in [
        (
            [
                Rule::literal("パスワード", "hunter2"),
                Rule::pattern("壊れた", "("),
            ],
            "hunter2",
        ),
        (
            [
                Rule::pattern("先", "(?P<x>s3cr3t)"),
                Rule::pattern("後", "(?P<x>b)"),
            ],
            "s3cr3t",
        ),
    ] {
        let err = Redactor::new(&rules).unwrap_err();
        let (debug, shown) = (format!("{err:?}"), err.to_string());
        assert!(!debug.contains(secret), "Debug leaked: {debug}");
        assert!(!shown.contains(secret), "Display leaked: {shown}");
        assert!(
            debug.contains("name") && debug.contains("reason"),
            "Debug shows the name and the reason: {debug}"
        );
    }
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

    errors_hide_other_rules();
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
