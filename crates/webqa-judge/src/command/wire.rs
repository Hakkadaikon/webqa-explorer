//! The request and answer lines of the bridge format, version 2.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};
use webqa_core::judge::{Answers, Image, ImageFormat, JudgeError, Question, QuestionId, Usage};

use super::PROTOCOL_VERSION;

/// One request line, written in this field order.
#[derive(Serialize)]
pub struct Request<'a> {
    version: u32,
    seq: u64,
    screen: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<WireImage>,
    questions: &'a [Question],
}

/// The image of a request, in a form no judge model owns.
#[derive(Serialize)]
pub struct WireImage {
    media_type: ImageFormat,
    /// The bytes in standard base64, padded, on one line.
    data: String,
}

/// The request line for `seq`, ending with a newline. Without `image` the
/// line has no `image` key.
pub fn request(seq: u64, screen: &str, image: Option<&Image>, questions: &[Question]) -> String {
    let request = Request {
        version: PROTOCOL_VERSION,
        seq,
        screen,
        image: image.map(wire_image),
        questions,
    };
    let mut line = serde_json::to_string(&request).unwrap_or_default();
    line.push('\n');
    line
}

fn wire_image(image: &Image) -> WireImage {
    WireImage {
        media_type: image.format(),
        data: STANDARD.encode(image.bytes()),
    }
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

/// The error the bridge reports. Its message is kept only when it is plain.
#[derive(Deserialize)]
pub struct ReplyError {
    kind: String,
    /// Any JSON: a message that is not a string is dropped, not malformed.
    #[serde(default)]
    message: Option<serde_json::Value>,
}

/// The longest message of a bridge's error that is kept.
const MAX_DETAIL: usize = 64;

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
    /// The error the bridge names. Any other kind is malformed.
    fn into_judge_error(self) -> JudgeError {
        match self.kind.as_str() {
            "api" => JudgeError::Api {
                detail: self.detail(),
            },
            "timeout" => JudgeError::Timeout,
            _ => JudgeError::Malformed,
        }
    }

    /// The message, when it is a plain string.
    fn detail(&self) -> Option<String> {
        let message = self.message.as_ref()?.as_str()?;
        plain(message).then(|| message.to_owned())
    }
}

/// Whether `message` is short, not only spaces, and only of letters,
/// digits, spaces and `._:-`, so that it holds no secret or page string
/// worth hiding.
fn plain(message: &str) -> bool {
    let allowed = |c: char| c.is_ascii_alphanumeric() || " ._:-".contains(c);
    said(message) && message.chars().all(allowed)
}

/// Whether `message` says something within the length kept.
fn said(message: &str) -> bool {
    message.len() <= MAX_DETAIL && !message.trim().is_empty()
}

/// A missing probability becomes NaN.
fn row((id, options): (String, Vec<Probability>)) -> (QuestionId, Vec<(String, f64)>) {
    let options = options
        .into_iter()
        .map(|o| (o.option, o.p.unwrap_or(f64::NAN)))
        .collect();
    (QuestionId::new(&id), options)
}

#[cfg(test)]
mod read_tests;
#[cfg(test)]
mod tests;
