//! Tests of the memory a report keeps.

use super::*;

/// A memory file with one element named `name`.
fn memory_named(name: &str) -> Value {
    json!({
        "schema": 1,
        "origin": "https://app.example.test",
        "screens": [{"elements": [{"role": "button", "name": name, "nth": 0}]}],
        "edges": [],
    })
}

/// The memory the run of [`report`] started from.
pub fn start_memory() -> Value {
    memory_named("保存")
}

#[test]
fn report_keeps_start_memory() {
    assert_eq!(report()["start_memory"], start_memory());
}

#[test]
fn start_memory_is_redacted() {
    let redactor = Redactor::new(&[Rule::literal("メールアドレス", "taro@example.test")]).unwrap();
    let memory = memory_named("taro@example.test");
    let report = build(
        &record(),
        ExitReason::BudgetExceeded,
        &meta(),
        &memory,
        &redactor,
    )
    .into_inner();
    assert_eq!(
        report["start_memory"],
        memory_named("[[メールアドレス]]"),
        "only the name of what is redacted"
    );
}
