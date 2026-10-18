//! Tests of finding the element a touch names, and the value to type in it.

use std::collections::BTreeMap;
use std::path::Path;

use url::Url;
use webqa_core::config::Secret;
use webqa_core::config::raw::parse;
use webqa_core::config::validate::validate;
use webqa_core::redact::{Redactor, Rule};
use webqa_core::screen::{Element, ElementId, normalize_name};

use super::*;

fn entry(role: &str, name: &str, node: &str) -> Entry {
    Entry {
        element: Element {
            role: role.to_owned(),
            name: name.to_owned(),
            value: None,
            rect: None,
        },
        node: node.to_owned(),
        in_dialog: false,
    }
}

fn id(role: &str, name: &str, nth: usize) -> ElementId {
    ElementId {
        role: role.to_owned(),
        name: name.to_owned(),
        nth,
    }
}

fn redactor() -> Redactor {
    Redactor::new(&[Rule::literal("メールアドレス", "taro@example.test")]).unwrap()
}

/// The ref of the entry `target` names, with the address masked.
fn found<'a>(target: &ElementId, entries: &'a [Entry]) -> Option<&'a str> {
    let url = Url::parse("https://app.example.test/home").unwrap();
    let entry = resolve(target, entries, &url, &redactor());
    entry.map(|e| e.node.as_str())
}

#[test]
fn resolves_target_through_redacted_ids() {
    let entries = [
        entry("button", "保存", "e1"),
        entry("textbox", "taro@example.test", "e2"),
        entry("button", "保存", "e3"),
        entry("button", "通知 (3)", "e4"),
    ];
    let masked = normalize_name(redactor().text("taro@example.test").as_inner());
    assert_eq!(
        [
            found(&id("textbox", &masked, 0), &entries),
            found(&id("textbox", "taro@example.test", 0), &entries),
            found(&id("button", "保存", 1), &entries),
            found(&id("button", "保存", 2), &entries),
            found(&id("button", "通知 (#)", 0), &entries),
        ],
        [Some("e2"), None, Some("e3"), None, Some("e4")],
        "ids made from the redacted, normalized names"
    );
    assert_eq!(found(&id("button", "保存", 0), &[]), None, "no elements");
}

/// The example configuration, cut down to what validation needs.
const CONFIG: &str = r#"
target:
  url: https://app.example.test
  allowed_origins: []
judge:
  kind: command
  command: ["./bridge/jev.py"]
  api_key: { env: JEV_API_KEY }
  timeout_secs: 30
  pricing: { input_per_1k: 0.0, output_per_1k: 0.0 }
limits:
  freeze_threshold: 3
  stuck_threshold: 3
  max_decisions: 200
  requests_per_hour: 600
  page_timeout_secs: 10
  back_confidence: 0.7
  error_confidence: 0.8
guard:
  forbidden: { keywords: [], selectors: [] }
  allowed: { selectors: [] }
  leave_paywall: back
redact: []
inputs:
  メールアドレス: { env: TEST_EMAIL }
"#;

fn inputs() -> BTreeMap<String, Secret> {
    let env = |name: &str| Some(format!("value-of-{name}"));
    let read = |_: &Path| Ok(String::new());
    validate(parse(CONFIG).unwrap(), &env, &read)
        .unwrap()
        .inputs
}

#[test]
fn input_value_by_accessible_name() {
    let inputs = inputs();
    let value = |name: &str| input_value(&entry("textbox", name, "e1"), &inputs);
    assert_eq!(
        [
            value("メールアドレス"),
            value("メールアドレス "),
            value("メール")
        ],
        [Some("value-of-TEST_EMAIL"), None, None],
        "the exact name only"
    );
}
