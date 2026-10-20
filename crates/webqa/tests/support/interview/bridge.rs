//! The bridge of a scene: the judge model through the product's command
//! judge, with a script that answers from a file and writes down each
//! request line it read.

use std::collections::VecDeque;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use serde_json::{Map, Value, json};
use tokio::sync::Notify;
use webqa_core::config;
use webqa_core::judge::{Answers, JudgeError};
use webqa_core::ports::{Clock, Judge, Pending};
use webqa_judge::table;

use crate::fakes::stuck::stuck;
use crate::scene::found;

/// The script of the bridge.
const SCRIPT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/bridge/answers.sh"
);

/// How many run directories were made by this process.
static RUNS: AtomicUsize = AtomicUsize::new(0);

/// A directory of one run, removed when dropped.
#[derive(Debug)]
pub struct RunDir(pub PathBuf);

impl RunDir {
    /// A new empty directory.
    pub fn new() -> Self {
        let n = RUNS.fetch_add(1, Ordering::SeqCst);
        let dir = env::temp_dir().join(format!("webqa-interview-{}-{n}", process::id()));
        let _ = fs::remove_dir_all(&dir);
        assert!(
            fs::create_dir_all(&dir).is_ok(),
            "the run directory is made"
        );
        Self(dir)
    }
}

impl Drop for RunDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The command of the bridge that declares `images` and answers from `dir`.
pub fn command(images: bool, dir: &Path) -> Vec<String> {
    let dir = dir.to_string_lossy().into_owned();
    vec![SCRIPT.to_owned(), images.to_string(), dir]
}

/// Writes `replies` to `dir` as the answer lines of the bridge.
pub fn write_answers(dir: &Path, replies: &VecDeque<Result<Answers, JudgeError>>) {
    let lines: Vec<String> = replies.iter().map(line).collect();
    let lines = format!("{}\n", lines.join("\n"));
    let wrote = fs::write(dir.join("answers.jsonl"), lines);
    assert!(wrote.is_ok(), "the answers are written");
}

/// One reply as the fields of an answer line, without the braces.
fn line(reply: &Result<Answers, JudgeError>) -> String {
    let fields = match reply {
        Ok(answers) => json!({
            "answers": table_of(answers),
            "usage": {
                "input_tokens": answers.usage.input_tokens,
                "output_tokens": answers.usage.output_tokens,
            },
        }),
        Err(error) => json!({ "error": { "kind": kind(error) } }),
    };
    let text = fields.to_string();
    text.get(1..text.len().saturating_sub(1))
        .unwrap_or_default()
        .to_owned()
}

/// The options of each question. A NaN probability is left out.
fn table_of(answers: &Answers) -> Map<String, Value> {
    let option = |(option, p): &(String, f64)| {
        if p.is_nan() {
            json!({ "option": option })
        } else {
            json!({ "option": option, "p": p })
        }
    };
    answers
        .table
        .iter()
        .map(|(id, options)| (id.as_str().to_owned(), options.iter().map(option).collect()))
        .collect()
}

/// The kind of error a bridge names for `error`. The others are not words
/// of the format, so the judge reads them as malformed: `Exhausted` and
/// `Bridge` come back as `Malformed` through the bridge.
const fn kind(error: &JudgeError) -> &'static str {
    match error {
        JudgeError::Api => "api",
        JudgeError::Timeout => "timeout",
        JudgeError::Exhausted | JudgeError::Malformed | JudgeError::Bridge => "malformed",
    }
}

/// The request lines the bridge read, in order.
pub fn asked(dir: &Path) -> Vec<Value> {
    let text = fs::read_to_string(dir.join("asked.jsonl")).unwrap_or_default();
    let read = |line: &str| found(serde_json::from_str(line).ok(), "a request line is JSON");
    text.lines().map(read).collect()
}

/// Starts the judge of the configuration, as the CLI does, with a bridge
/// that has `answers` answers. Once they are used, the wait for the next
/// one wakes `gate`.
pub async fn start(judge: &config::Judge, answers: usize, gate: Arc<Notify>) -> Box<dyn Judge> {
    let clock = BridgeClock {
        gate,
        answers,
        waits: 0,
    };
    let started = table::start(judge, Box::new(clock)).await;
    found(started.ok(), "the bridge starts")
}

/// The clock of the bridge. The judge waits once for the version line and
/// once for each answer. Those waits never end, so the line comes first;
/// the wait for an answer the bridge does not have wakes the gate.
struct BridgeClock {
    gate: Arc<Notify>,
    answers: usize,
    waits: usize,
}

impl Clock for BridgeClock {
    fn elapsed(&self) -> Duration {
        Duration::ZERO
    }

    fn sleep_until(&mut self, _at: Duration) -> Pending<'_, ()> {
        self.waits += 1;
        if self.waits > self.answers + 1 {
            self.gate.notify_one();
        }
        stuck()
    }
}
