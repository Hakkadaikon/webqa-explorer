//! Tests of reading the message of a bridge's error.

use super::*;

/// What reading an `api` error with `message` gives.
fn api_error(message: &str) -> Result<Option<Answers>, JudgeError> {
    let error = serde_json::json!({"kind": "api", "message": message});
    let line = serde_json::json!({"version": 2, "seq": 1, "error": error});
    read(line.to_string().as_bytes(), 1)
}

/// An API error with `detail`.
fn api(detail: Option<&str>) -> Result<Option<Answers>, JudgeError> {
    Err(JudgeError::Api {
        detail: detail.map(str::to_owned),
    })
}

#[test]
fn an_api_error_keeps_a_plain_message() {
    assert_eq!(api_error("http 500"), api(Some("http 500")));
    let plain = "Connection_reset-by.peer: 104";
    assert_eq!(
        api_error(plain),
        api(Some(plain)),
        "every allowed character"
    );
}

#[test]
fn a_message_with_other_characters_is_dropped() {
    for message in [
        "quota for taro@example.test",
        "残高が足りない",
        "a/b",
        "",
        "x\ny",
    ] {
        assert_eq!(api_error(message), api(None), "{message:?}");
    }
    let line = br#"{"version":2,"seq":1,"error":{"kind":"api"}}"#;
    assert_eq!(read(line, 1), api(None), "no message");
    let number = br#"{"version":2,"seq":1,"error":{"kind":"api","message":500}}"#;
    assert_eq!(read(number, 1), api(None), "a message that is not a string");
}

#[test]
fn a_long_message_is_dropped() {
    let longest = "a".repeat(64);
    assert_eq!(api_error(&longest), api(Some(&longest)), "64 characters");
    assert_eq!(api_error(&"a".repeat(65)), api(None), "65 characters");
}
