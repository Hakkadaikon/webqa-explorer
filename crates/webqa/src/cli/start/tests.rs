//! Tests of the memory a replay starts from.

use serde_json::json;
use std::collections::BTreeSet;
use url::Url;
use webqa_core::explore::ElementOp;
use webqa_core::redact::Redactor;
use webqa_core::screen::{Element, ElementId, Screen};

use super::*;

const ORIGIN: &str = "https://app.example.test";

#[test]
fn broken_start_memory_is_config_error() {
    let secret = "taro@example.test";
    let broken = [json!({"schema": 9, "note": secret}), json!(secret)];
    for kept in broken {
        let stop = given(ORIGIN, Some(kept)).err().map(Stop::into_parts);
        assert_eq!(
            stop,
            Some((2, "実行結果を再実行の台本として読めない".to_owned())),
            "a fixed sentence without the memory"
        );
    }
}

#[test]
fn start_memory_is_given_or_empty() {
    let kept = Memory::default().to_json(ORIGIN);
    let from = |kept| given(ORIGIN, kept).unwrap();
    for start in [from(Some(kept)), from(None)] {
        let Start::Given(memory) = start else {
            panic!("a replay does not start from the stored memory")
        };
        assert_eq!(memory.screen_count(), 0);
    }
}

#[test]
fn start_memory_of_another_origin_is_empty() {
    let kept = Memory::default().to_json("https://other.example.test");
    let Ok(Start::Given(memory)) = given(ORIGIN, Some(kept)) else {
        panic!("a memory of another origin starts a replay empty")
    };
    assert_eq!(memory.screen_count(), 0);
}

/// A memory with a screen of one button, operated.
fn saved() -> Memory {
    let element = Element {
        role: "button".to_owned(),
        name: "保存".to_owned(),
        value: None,
        rect: None,
    };
    let url = Url::parse("https://app.example.test/").unwrap();
    let screen = Redactor::default().screen(Screen {
        url,
        elements: vec![element],
    });
    let mut memory = Memory::default();
    let at = memory.visit(&screen, &BTreeSet::new());
    let id = ElementId {
        role: "button".to_owned(),
        name: "保存".to_owned(),
        nth: 0,
    };
    memory.record(at, &id, ElementOp::Click, at);
    memory
}

#[test]
fn given_keeps_the_start_memory_of_the_report() {
    let kept = saved().to_json(ORIGIN);
    let Ok(Start::Given(memory)) = given(ORIGIN, Some(kept.clone())) else {
        panic!("the kept memory starts the replay")
    };
    assert_eq!(
        (memory.screen_count(), memory.to_json(ORIGIN)),
        (1, kept),
        "the memory of the report, not an empty one"
    );
}

const PERSONA: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/docs/persona.yaml"
);

#[test]
fn an_interview_setup_reads_secrets_from_the_environment() {
    let yaml = format!(
        "
target: {{ url: 'http://localhost:1/', allowed_origins: [] }}
judge:
  kind: command
  command: ['x']
  api_key: {{ env: PATH }}
  timeout_secs: 1
  pricing: {{ input_per_1k: 0.0, output_per_1k: 0.0 }}
limits:
  freeze_threshold: 1
  stuck_threshold: 1
  max_decisions: 1
  requests_per_hour: 2
  page_timeout_secs: 1
  back_confidence: 0.7
  error_confidence: 0.8
guard:
  forbidden: {{ keywords: [], selectors: [] }}
  allowed: {{ selectors: [] }}
interview:
  persona: {PERSONA}
  trials: 1
"
    );
    let (setup, _) = interview_setup(&yaml).unwrap();
    assert!(
        setup.judge.api_key.is_some(),
        "the key came from the environment"
    );
}
