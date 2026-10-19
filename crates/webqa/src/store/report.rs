//! The report of a run: the issues, the operations, the judge's replies as a
//! script to replay, the timings, the usage and its cost, and why it ended.

mod sections;
#[cfg(test)]
pub(crate) mod tests;

use serde_json::{Value, json};
use webqa_core::event::{Event, Mode};
use webqa_core::judge::Usage;
use webqa_core::machine::ExitReason;
use webqa_core::redact::{Redacted, Redactor};

use crate::record::Record;

/// The version of the format.
pub const SCHEMA: u32 = 1;

/// Prices per 1000 tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Price {
    /// The price of 1000 tokens sent.
    pub input_per_1k: f64,
    /// The price of 1000 tokens received.
    pub output_per_1k: f64,
}

/// What the report says about the run besides the record.
#[derive(Debug, Clone)]
pub struct Meta {
    /// The kind of run.
    pub mode: Mode,
    /// Wall-clock start, as in `run_started`.
    pub started_at: Option<String>,
    /// The start URL, redacted.
    pub target: Redacted<String>,
    /// The keys of `inputs`; an input whose element has one of these names
    /// is written as that key.
    pub inputs: Vec<String>,
    /// `None` leaves the cost empty.
    pub price: Option<Price>,
}

/// The report of a run that ended for `reason`, through the redactor.
#[must_use]
pub fn build(
    record: &Record,
    reason: ExitReason,
    meta: &Meta,
    redactor: &Redactor,
) -> Redacted<Value> {
    let report = json!({
        "schema": SCHEMA,
        "mode": meta.mode,
        "started_at": meta.started_at,
        "target": meta.target.as_inner(),
        "reason": reason_name(reason),
        "stuck": reason.stuck_kind(),
        "usage": sections::usage_section(record, meta.price),
        "steps": sections::steps_section(&record.acts, &meta.inputs),
        "calls": sections::calls_section(&record.calls),
        "issues": sections::issues_section(&record.found),
    });
    redactor.json(report)
}

/// The name of `reason`, as the last line of the event log writes it.
fn reason_name(reason: ExitReason) -> Value {
    let line = serde_json::to_value(Event::finished(reason)).unwrap_or_default();
    line.get("reason").cloned().unwrap_or_default()
}

/// The cost of `usage` at `price`. `None` without a price.
#[must_use]
pub fn cost(usage: Usage, price: Option<Price>) -> Option<f64> {
    price.map(|price| priced(usage, price))
}

/// The cost of `usage` at `price`.
fn priced(usage: Usage, price: Price) -> f64 {
    let input = thousands(usage.input_tokens) * price.input_per_1k;
    thousands(usage.output_tokens).mul_add(price.output_per_1k, input)
}

/// The most tokens counted in one sum, `u32::MAX`.
const MAX_TOKENS: f64 = 4_294_967_295.0;

/// `tokens` in thousands.
fn thousands(tokens: u64) -> f64 {
    u32::try_from(tokens).map_or(MAX_TOKENS, f64::from) / 1000.0
}
