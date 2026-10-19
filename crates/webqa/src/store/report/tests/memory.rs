//! Tests of the memory a report keeps.

use std::collections::BTreeSet;

use url::Url;
use webqa_core::explore::Memory;
use webqa_core::screen::{Element, Screen};

use super::*;

/// A memory file with `保存` renamed `name`.
fn memory_named(name: &str) -> Value {
    let mut memory = Memory::default();
    let element = Element {
        role: "button".to_owned(),
        name: name.to_owned(),
        value: None,
        rect: None,
    };
    let screen = Redactor::default().screen(Screen {
        url: Url::parse("https://app.example.test/").unwrap(),
        elements: vec![element],
    });
    memory.visit(&screen, &BTreeSet::new());
    memory.to_json("https://app.example.test")
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
