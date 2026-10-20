//! Tests of reading the YAML, and the example shared with validation.

use std::mem::discriminant;

use super::*;

const ARCHITECTURE: &str = include_str!("../../../../../docs/arch/config.md");

/// The YAML example of `docs/arch/config.md`, as written there.
pub fn example() -> &'static str {
    let start = ARCHITECTURE.find("```yaml\ntarget:").unwrap() + "```yaml\n".len();
    let rest = &ARCHITECTURE[start..];
    &rest[..rest.find("```").unwrap()]
}

/// The example with `from` replaced by `to`. Fails if `from` is absent.
pub fn changed(from: &str, to: &str) -> String {
    let yaml = example().replacen(from, to, 1);
    assert_ne!(yaml, example(), "the example has no {from:?}");
    yaml
}

#[test]
fn reads_target_inputs_and_stages() {
    let raw = parse(example()).unwrap();
    assert_eq!(raw.target.url, "https://app.example.test", "target url");
    let email = raw.inputs.get("メールアドレス").unwrap();
    assert_eq!(email.env, "TEST_EMAIL", "input env");
    let stages = raw.scenario.unwrap().stages;
    assert_eq!(stages.len(), 1, "one stage");
    let login = &stages[0];
    assert_eq!(login.name, "ログイン", "stage name");
    assert_eq!(
        login.done_when.url.as_deref(),
        Some("/home"),
        "done_when url"
    );
    let element = login.done_when.element.as_ref().unwrap();
    assert_eq!(element.role, "button", "element role");
    assert_eq!(element.name, "ログアウト", "element name");
    let all = [
        Op::Click,
        Op::Input,
        Op::ScrollDown,
        Op::ScrollUp,
        Op::Back,
        Op::Enter,
        Op::Wait,
        Op::Done,
        Op::Stuck,
    ];
    assert_eq!(raw.ops, all, "all nine ops when omitted");
    reads_two_stages_in_order();
    reads_snake_case_ops();
}

fn reads_snake_case_ops() {
    let yaml = format!("{}ops: [click, scroll_down, scroll_up, back]\n", example());
    let ops = parse(&yaml).unwrap().ops;
    let want = [Op::Click, Op::ScrollDown, Op::ScrollUp, Op::Back];
    assert_eq!(ops, want, "snake_case names");
}

fn reads_two_stages_in_order() {
    let two = format!(
        "{}    - name: 注文\n      goal: 注文を済ませる\n      done_when:\n        text: 注文完了\n",
        changed("  leave_paywall: back\n", "")
    );
    let raw = parse(&two).unwrap();
    let stages = raw.scenario.unwrap().stages;
    let names: Vec<_> = stages.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["ログイン", "注文"], "stages in order");
    let text = stages[1].done_when.text.as_deref();
    assert_eq!(text, Some("注文完了"), "second done_when text");
    assert_eq!(raw.guard.leave_paywall, "back", "leave_paywall default");
}

#[test]
fn rejects_tls_verification_key() {
    let judge = changed(
        "  timeout_secs: 30\n",
        "  timeout_secs: 30\n  tls_verify: false\n",
    );
    let err = parse(&judge).unwrap_err();
    assert!(
        err.to_string().contains("tls_verify"),
        "unknown field: {err}"
    );
    let target = changed(
        "  url: https://app.example.test\n",
        "  url: https://app.example.test\n  insecure: true\n",
    );
    let err = parse(&target).unwrap_err();
    assert!(err.to_string().contains("insecure"), "unknown field: {err}");
    rejects_unknown_fields_in_nested_forms();
}

/// Forms written inline, including the ones read as one of two shapes.
const NESTED_UNKNOWN: [(&str, &str); 3] = [
    ("pattern: \"[", "tls_verify: false, pattern: \"["),
    ("pattern: \"[", "env: TEST_ID, pattern: \"["),
    ("{ env: TEST_EMAIL }", "{ env: TEST_EMAIL, extra: 1 }"),
];

fn rejects_unknown_fields_in_nested_forms() {
    let yaml_error = parse("[").unwrap_err();
    for (from, to) in NESTED_UNKNOWN {
        let err = parse(&changed(from, to)).unwrap_err();
        assert_eq!(discriminant(&err), discriminant(&yaml_error), "{to}: {err}");
    }
}

/// Every `api_key` but `{ env: <name> }`, including a mapping that names
/// the key itself.
const NOT_ENV_REF: [&str; 4] = [
    "\"sk-test-123456\"",
    "{ sk-test-123456 }",
    "{ env: [sk-test-123456] }",
    "{ env: JEV_API_KEY, tls_verify: false }",
];

#[test]
fn literal_api_key_rejected_without_value() {
    for form in NOT_ENV_REF {
        let yaml = changed("api_key: { env: JEV_API_KEY }", &format!("api_key: {form}"));
        let err = parse(&yaml).unwrap_err();
        let want = discriminant(&ConfigError::LiteralApiKey);
        assert_eq!(discriminant(&err), want, "{form}: {err:?}");
        assert_hides_key(&err);
    }
    // Broken YAML next to the key: the error must not quote the lines.
    let broken = changed("  timeout_secs: 30\n", "  timeout_secs: [\n").replacen(
        "{ env: JEV_API_KEY }",
        "\"sk-test-123456\"",
        1,
    );
    assert_hides_key(&parse(&broken).unwrap_err());
}

fn assert_hides_key(err: &ConfigError) {
    assert!(
        !err.to_string().contains("sk-test-123456"),
        "Display hides the key: {err}"
    );
    assert!(
        !format!("{err:?}").contains("sk-test-123456"),
        "Debug hides the key"
    );
}
