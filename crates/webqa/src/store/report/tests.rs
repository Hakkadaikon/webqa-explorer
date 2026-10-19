//! Tests of the report of a run: its shape, its cost and what it leaves out.

use std::collections::BTreeMap;
use std::time::Duration;

use serde_json::json;
use webqa_core::issue::{Issue, IssueKind};
use webqa_core::judge::{Answers, JudgeError, QuestionId};
use webqa_core::machine::{Cause, StuckKind};
use webqa_core::redact::Rule;
use webqa_core::screen::ElementId;

use super::*;
use crate::record::{Act, Call, Found};

/// The answers of the first call: an input on element 1, an error shown.
pub fn first_answers() -> Answers {
    let table = BTreeMap::from([
        (
            QuestionId::operation(),
            vec![("input".to_owned(), 0.9), ("back".to_owned(), 0.1)],
        ),
        (QuestionId::target(), vec![("1".to_owned(), 0.8)]),
        (QuestionId::error_shown(), vec![("true".to_owned(), 0.9)]),
    ]);
    Answers {
        table,
        usage: Usage {
            input_tokens: 120,
            output_tokens: 3,
        },
    }
}

fn act(step: u64, op: &str, target: Option<ElementId>, cause: Cause, ms: [u64; 2]) -> Act {
    let [act, settle] = ms;
    Act {
        step,
        op: op.to_owned(),
        target,
        p: if cause == Cause::Judge { 0.9 } else { 1.0 },
        cause,
        act: Duration::from_millis(act),
        settle: Duration::from_millis(settle),
    }
}

fn textbox(name: &str) -> ElementId {
    ElementId {
        role: "textbox".to_owned(),
        name: name.to_owned(),
        nth: 0,
    }
}

/// Two calls, an input and a reload, and an error shown at step 1.
pub fn record() -> Record {
    let calls = vec![
        Call {
            step: 1,
            reply: Ok(first_answers()),
            think: Duration::from_millis(1200),
        },
        Call {
            step: 2,
            reply: Err(JudgeError::Timeout),
            think: Duration::from_millis(800),
        },
    ];
    let acts = vec![
        act(
            1,
            "input",
            Some(textbox("メールアドレス")),
            Cause::Judge,
            [300, 500],
        ),
        act(3, "reload", None, Cause::Frozen, [100, 200]),
    ];
    let issue = Issue {
        kind: IssueKind::ErrorDisplay,
        evidence: None,
        label: None,
        p: Some(0.9),
    };
    let found = vec![Found {
        step: 1,
        issue,
        shot: Some("ISSUE_error_display_step1.png".to_owned()),
    }];
    Record { calls, acts, found }
}

/// The meta of an exploration of the app with one input and a price.
pub fn meta() -> Meta {
    Meta {
        mode: Mode::Explore,
        started_at: Some("2026-10-18T02:00:00Z".to_owned()),
        target: Redactor::default().text("https://app.example.test/"),
        inputs: vec!["メールアドレス".to_owned()],
        price: Some(Price {
            input_per_1k: 0.001,
            output_per_1k: 0.002,
        }),
    }
}

/// The report of [`record`] for a run out of budget.
pub fn report() -> Value {
    build(
        &record(),
        ExitReason::BudgetExceeded,
        &meta(),
        &start_memory(),
        &Redactor::default(),
    )
    .into_inner()
}

/// The values of the report checked one by one besides the snapshot.
fn main_values(report: &Value) -> [&Value; 8] {
    [
        &report["reason"],
        &report["stuck"],
        &report["steps"][0]["input"],
        &report["steps"][1]["input"],
        &report["steps"][1]["cause"],
        &report["calls"][1]["error"],
        &report["calls"][1]["answers"],
        &report["issues"][0]["screenshot"],
    ]
}

#[test]
fn report_matches_snapshot() {
    let report = report();
    insta::assert_snapshot!(serde_json::to_string_pretty(&report).unwrap());
    let cost = report["usage"]["cost"].as_f64().unwrap();
    assert!((cost - 0.000_126).abs() < 1e-12, "the cost: {cost}");
    let mut usage = report["usage"].clone();
    usage["cost"] = json!(0.000_126);
    let tokens = json!({"calls": 2, "input_tokens": 120, "output_tokens": 3, "cost": 0.000_126});
    assert_eq!(usage, tokens, "the usage");
    let expected = [
        &json!("budget_exceeded"),
        &Value::Null,
        &json!("メールアドレス"),
        &Value::Null,
        &json!("frozen"),
        &json!("timeout"),
        &Value::Null,
        &json!("screenshots/ISSUE_error_display_step1.png"),
    ];
    assert_eq!(main_values(&report), expected, "the main values");
}

#[test]
fn cost_from_usage_and_pricing() {
    let price = |input_per_1k, output_per_1k| {
        Some(Price {
            input_per_1k,
            output_per_1k,
        })
    };
    let cases = [
        ((1500, 500), price(0.003, 0.015), Some(0.012)),
        ((0, 0), price(0.003, 0.015), Some(0.0)),
        ((1000, 0), price(0.0, 0.0), Some(0.0)),
        ((1500, 500), None, None),
    ];
    for ((input_tokens, output_tokens), price, expected) in cases {
        let usage = Usage {
            input_tokens,
            output_tokens,
        };
        let got = cost(usage, price);
        let close = got.zip(expected).map(|(a, b)| (a - b).abs() < 1e-12);
        assert_eq!(
            (got.is_some(), close.unwrap_or(true)),
            (expected.is_some(), true),
            "{usage:?} at {price:?}: {got:?}, not {expected:?}"
        );
    }
}

#[test]
fn report_names_redacted_and_values_left_out() {
    let redactor = Redactor::new(&[Rule::literal("メールアドレス", "taro@example.test")]).unwrap();
    let mut record = record();
    let name = redactor.text("taro@example.test").into_inner();
    record.acts[0].target = Some(textbox(&name));
    let mut answers = first_answers();
    answers.table.insert(
        QuestionId::new("note"),
        vec![("連絡先は taro@example.test".to_owned(), 0.5)],
    );
    record.calls[0].reply = Ok(answers);
    let report = build(
        &record,
        ExitReason::BudgetExceeded,
        &meta(),
        &start_memory(),
        &redactor,
    )
    .into_inner();
    let text = report.to_string();
    assert_eq!(
        (text.contains("taro@example.test"), name.as_str()),
        (false, "[[メールアドレス]]"),
        "no value in the report: {text}"
    );
    assert_eq!(
        report["steps"][0]["input"],
        Value::Null,
        "a name that is not a key of the inputs"
    );
}

#[test]
fn stuck_kind_in_report() {
    let report = build(
        &record(),
        ExitReason::Stuck(StuckKind::NoProgress),
        &meta(),
        &start_memory(),
        &Redactor::default(),
    )
    .into_inner();
    assert_eq!(
        (&report["reason"], &report["stuck"]),
        (&json!("stuck"), &json!("no_progress")),
        "the reason and the kind"
    );
}

mod edges;
mod memory;

pub use self::memory::start_memory;
