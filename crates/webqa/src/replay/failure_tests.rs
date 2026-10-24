//! Tests of reading the failures a report names back into the judge's.

use serde_json::{Value, json};

use super::tests::screen;
use super::*;
use crate::store::report::tests::report;

/// A report whose calls each failed as `names` say, in order.
fn failing(names: &[&str]) -> String {
    let mut report = report();
    let calls: Vec<Value> = names
        .iter()
        .map(
            |name| json!({"step": 1, "think_ms": 0, "error": name, "answers": null, "usage": null}),
        )
        .collect();
    report["calls"] = Value::from(calls);
    report.to_string()
}

/// The replies of replaying `text`, one per call.
async fn replayed(text: &str, calls: usize) -> Vec<Result<Answers, JudgeError>> {
    let (_, mut replay, _) = Replay::from_report(text).unwrap();
    let screen = screen();
    let mut replies = Vec::new();
    for _ in 0..calls {
        replies.push(replay.ask(&screen, &[]).await);
    }
    replies
}

#[tokio::test]
async fn each_failure_name_reads_back_as_its_judge_error() {
    let names = ["exhausted", "api", "timeout", "malformed", "bridge"];
    let expected = vec![
        Err(JudgeError::Exhausted),
        Err(JudgeError::Api { detail: None }),
        Err(JudgeError::Timeout),
        Err(JudgeError::Malformed),
        Err(JudgeError::Bridge),
    ];
    assert_eq!(replayed(&failing(&names), 5).await, expected, "row by row");
    let reversed: Vec<&str> = names.iter().rev().copied().collect();
    let back: Vec<_> = expected.into_iter().rev().collect();
    assert_eq!(replayed(&failing(&reversed), 5).await, back, "in any order");
}

#[test]
fn an_unknown_failure_name_is_not_a_report() {
    let read = Replay::from_report(&failing(&["quota"])).map(|(mode, ..)| mode);
    assert_eq!(read, Err(ReplayError), "not one of the names");
}
