//! The sections of the report: the usage, the operations, the judge calls and
//! the issues.

use std::time::Duration;

use serde_json::{Map, Value, json};
use webqa_core::judge::Answers;
use webqa_core::redact::Redacted;
use webqa_core::screen::ElementId;

use super::{Price, cost};
use crate::record::{Act, Call, Found, Record};
use crate::store::screenshots::DIR;

/// The number of calls, the tokens and their cost.
pub fn usage_section(record: &Record, price: Option<Price>) -> Value {
    let usage = record.usage();
    json!({
        "calls": record.calls.len(),
        "input_tokens": usage.input_tokens,
        "output_tokens": usage.output_tokens,
        "cost": cost(usage, price),
    })
}

/// Each operation, in order.
pub fn steps_section(acts: &[Act], inputs: &[String]) -> Value {
    acts.iter().map(|act| step(act, inputs)).collect()
}

/// Each judge call, in order.
pub fn calls_section(calls: &[Call]) -> Value {
    calls.iter().map(call).collect()
}

/// Each issue, in order.
pub fn issues_section(found: &[Found]) -> Value {
    found.iter().map(issue).collect()
}

fn step(act: &Act, inputs: &[String]) -> Value {
    json!({
        "step": act.step,
        "cause": act.cause,
        "op": act.op,
        "target": act.target.as_ref().map(element),
        "input": input_key(act, inputs),
        "p": act.p,
        "act_ms": millis(act.act),
        "settle_ms": millis(act.settle),
    })
}

fn element(id: &ElementId) -> Value {
    json!({ "role": id.role, "name": id.name, "nth": id.nth })
}

/// The key of `inputs` an input went to, when its element is named after one.
/// The value typed is never written.
fn input_key<'a>(act: &'a Act, inputs: &[String]) -> Option<&'a str> {
    let name = act.target.as_ref().map(|target| target.name.as_str());
    name.filter(|name| act.op == "input" && inputs.iter().any(|key| key == name))
}

fn millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn call(call: &Call) -> Value {
    let (answers, error, usage) = match &call.reply {
        Ok(answers) => (json!(answers.table), Value::Null, usage(answers)),
        Err(error) => (Value::Null, json!(error), Value::Null),
    };
    json!({
        "step": call.step,
        "think_ms": millis(call.think),
        "error": error,
        "answers": answers,
        "usage": usage,
    })
}

fn usage(answers: &Answers) -> Value {
    json!({
        "input_tokens": answers.usage.input_tokens,
        "output_tokens": answers.usage.output_tokens,
    })
}

fn issue(found: &Found) -> Value {
    let issue = &found.issue;
    let fields = [
        ("step", json!(found.step)),
        ("kind", json!(issue.kind)),
        ("class", json!(issue.class())),
        (
            "evidence",
            json!(issue.evidence.as_ref().map(Redacted::as_inner)),
        ),
        ("label", json!(issue.label)),
        ("p", json!(issue.p)),
    ];
    let named = fields
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value));
    let shot = Some(screenshot(&found.shot));
    let entry: Map<String, Value> = named.chain(shot).collect();
    Value::Object(entry)
}

/// The field of the screenshot `shot`, left out of an issue that has none.
fn screenshot(shot: &str) -> (String, Value) {
    ("screenshot".to_owned(), json!(format!("{DIR}/{shot}")))
}
