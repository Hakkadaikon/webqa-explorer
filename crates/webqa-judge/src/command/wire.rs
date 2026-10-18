//! The lines of the bridge format, version 1.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use webqa_core::judge::{Answers, JudgeError, Question, QuestionId, Usage};

use super::PROTOCOL_VERSION;

/// One request line, written in this field order.
#[derive(Serialize)]
pub struct Request<'a> {
    version: u32,
    seq: u64,
    screen: &'a str,
    questions: &'a [Question],
}

/// The request line for `seq`, ending with a newline.
pub fn request(seq: u64, screen: &str, questions: &[Question]) -> String {
    let request = Request {
        version: PROTOCOL_VERSION,
        seq,
        screen,
        questions,
    };
    let mut line = serde_json::to_string(&request).unwrap_or_default();
    line.push('\n');
    line
}

/// One answer line. Unknown fields are ignored.
#[derive(Deserialize)]
pub struct Reply {
    version: u32,
    seq: u64,
    error: Option<ReplyError>,
    answers: Option<BTreeMap<String, Vec<Probability>>>,
    #[serde(default)]
    usage: ReplyUsage,
}

/// The error the bridge reports. Its message is never read.
#[derive(Deserialize)]
pub struct ReplyError {
    kind: String,
}

/// One option with its probability, which may be missing.
#[derive(Deserialize)]
pub struct Probability {
    option: String,
    #[serde(default)]
    p: Option<f64>,
}

/// Tokens used, `0` and `0` when omitted.
#[derive(Default, Deserialize)]
pub struct ReplyUsage {
    input_tokens: u64,
    output_tokens: u64,
}

/// The errors a bridge may name. Any other kind is malformed.
const ERRORS: [(&str, JudgeError); 2] =
    [("api", JudgeError::Api), ("timeout", JudgeError::Timeout)];

/// Reads an answer line to the request `seq`. `None` for a late answer to
/// an earlier request, which the caller skips. A line that is not UTF-8 is
/// malformed.
pub fn read(line: &[u8], seq: u64) -> Result<Option<Answers>, JudgeError> {
    let reply = parse(line)?;
    on_time(reply.seq, seq)?
        .then(|| reply.into_answers())
        .transpose()
}

fn parse(line: &[u8]) -> Result<Reply, JudgeError> {
    serde_json::from_slice::<Reply>(line)
        .ok()
        .filter(|reply| reply.version == PROTOCOL_VERSION)
        .ok_or(JudgeError::Malformed)
}

/// `true` for the answer to `want`, `false` for one to an earlier request.
fn on_time(got: u64, want: u64) -> Result<bool, JudgeError> {
    match got.cmp(&want) {
        Ordering::Less => Ok(false),
        Ordering::Equal => Ok(true),
        Ordering::Greater => Err(JudgeError::Malformed),
    }
}

impl Reply {
    fn into_answers(self) -> Result<Answers, JudgeError> {
        let usage = Usage {
            input_tokens: self.usage.input_tokens,
            output_tokens: self.usage.output_tokens,
        };
        match (self.error, self.answers) {
            (Some(error), _) => Err(error.into_judge_error()),
            (None, Some(table)) => Ok(Answers {
                table: table.into_iter().map(row).collect(),
                usage,
            }),
            (None, None) => Err(JudgeError::Malformed),
        }
    }
}

impl ReplyError {
    fn into_judge_error(self) -> JudgeError {
        ERRORS
            .into_iter()
            .find(|(kind, _)| *kind == self.kind)
            .map_or(JudgeError::Malformed, |(_, error)| error)
    }
}

/// A missing probability becomes NaN.
fn row((id, options): (String, Vec<Probability>)) -> (QuestionId, Vec<(String, f64)>) {
    let options = options
        .into_iter()
        .map(|o| (o.option, o.p.unwrap_or(f64::NAN)))
        .collect();
    (QuestionId::new(&id), options)
}
