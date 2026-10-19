//! Tests of the memory a replay starts from.

use serde_json::json;

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
