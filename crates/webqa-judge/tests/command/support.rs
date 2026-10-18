//! The clock, the screens and the answers the contract tests share.

use std::collections::VecDeque;
use std::error::Error;
use std::fs;
use std::future;
use std::time::Duration;

use tokio::process::Command;

use serde_json::{Value, json};
use url::{ParseError, Url};
use webqa_core::judge::{Answers, JudgeError, QuestionId, Usage};
use webqa_core::ports::{Clock, Judge, Pending};
use webqa_core::redact::{Redacted, Redactor};
use webqa_core::screen::{Element, Rect, Screen, ScreenText, Viewport, screen_text};
use webqa_judge::command::{Bridge, CommandJudge};

const SCRIPT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/bridge.sh");

/// Each wait takes the next of the script: done at once when `true`, and
/// done after [`REAL_WAIT`] of real time when `false` or when the script
/// has run out. A judge that never answers then fails the test with a
/// timeout instead of stopping it.
struct Clockwork(VecDeque<bool>);

/// How long a wait that is not cut short takes, in seconds of real time.
const REAL_WAIT: &str = "5";

impl Clock for Clockwork {
    fn elapsed(&self) -> Duration {
        Duration::ZERO
    }

    fn sleep_until(&mut self, _at: Duration) -> Pending<'_, ()> {
        if self.0.pop_front().unwrap_or(false) {
            Box::pin(future::ready(()))
        } else {
            Box::pin(real_wait())
        }
    }
}

/// Waits [`REAL_WAIT`] seconds of real time by running `sleep`, which is
/// killed when the wait is dropped.
async fn real_wait() {
    let sleep = Command::new("sleep")
        .arg(REAL_WAIT)
        .kill_on_drop(true)
        .status();
    sleep.await.ok();
}

/// A clock that follows `script`.
pub fn clock(script: &[bool]) -> Box<dyn Clock> {
    Box::new(Clockwork(script.iter().copied().collect()))
}

/// A judge on the bridge, with an empty screen to ask about.
pub struct Bench {
    /// The judge under test.
    pub judge: CommandJudge,
    empty: Redacted<ScreenText>,
}

impl Bench {
    /// Asks no question about the empty screen.
    pub async fn ask(&mut self) -> Result<Answers, JudgeError> {
        self.judge.ask(&self.empty, &[]).await
    }
}

/// The bridge run with `args`, the key and a clock with `script`.
pub fn bench(
    args: &[&str],
    api_key: Option<&str>,
    script: &[bool],
) -> Result<Bench, Box<dyn Error>> {
    let command: Vec<String> = ["sh", SCRIPT]
        .iter()
        .chain(args)
        .map(|&a| a.to_owned())
        .collect();
    let bridge = Bridge {
        command: &command,
        api_key,
        timeout: Duration::from_secs(30),
    };
    Ok(Bench {
        judge: CommandJudge::spawn(&bridge, clock(script))?,
        empty: text(Vec::new())?,
    })
}

/// The bridge in `mode`, with a clock that ends the wait only after
/// [`REAL_WAIT`] of real time.
pub fn judge(mode: &str) -> Result<Bench, Box<dyn Error>> {
    bench(&[mode], None, &[])
}

fn text(elements: Vec<Element>) -> Result<Redacted<ScreenText>, ParseError> {
    let screen = Screen {
        url: Url::parse("https://app.example.test/")?,
        elements,
    };
    let viewport = Viewport {
        width: 800,
        height: 600,
    };
    Ok(screen_text(&Redactor::default().screen(screen), viewport))
}

/// A screen of one button "保存", whose text is `1: button "保存"`.
pub fn save_button() -> Result<Redacted<ScreenText>, ParseError> {
    let rect = Rect {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    text(vec![Element {
        role: "button".to_owned(),
        name: "保存".to_owned(),
        value: None,
        rect: Some(rect),
    }])
}

/// The options of `id` in the answers.
pub fn options(answers: &Answers, id: &str) -> Option<Vec<(String, f64)>> {
    answers.table.get(&QuestionId::new(id)).cloned()
}

/// Owned options.
pub fn pairs(table: &[(&str, f64)]) -> Vec<(String, f64)> {
    table.iter().map(|&(o, p)| (o.to_owned(), p)).collect()
}

/// The fixed answers of the `answer` mode.
pub fn fixed() -> Answers {
    let rows: [(&str, &[(&str, f64)]); 4] = [
        ("operation", &[("click", 0.9), ("back", 0.1)]),
        ("target", &[("1", 0.8), ("2", 0.6)]),
        ("error_shown", &[("true", 0.1)]),
        ("goal_reached", &[("true", 0.0)]),
    ];
    Answers {
        table: rows
            .iter()
            .map(|&(id, t)| (QuestionId::new(id), pairs(t)))
            .collect(),
        usage: Usage {
            input_tokens: 120,
            output_tokens: 3,
        },
    }
}

/// A file under the target's temporary directory, removed if it is there.
pub fn fresh_log(name: &str) -> String {
    let path = format!("{}/{name}", env!("CARGO_TARGET_TMPDIR"));
    let _ = fs::remove_file(&path);
    path
}

/// The lines the `answer` mode recorded, as JSON.
pub fn logged(path: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    let lines = fs::read_to_string(path)?;
    Ok(lines
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?)
}

/// The two requests of the round trip: the screen with one button and the
/// three questions of a step without a goal, then the empty screen with no
/// question.
pub fn round_trip_requests() -> [Value; 2] {
    let first = json!({"version": 1, "seq": 1, "screen": "1: button \"保存\"", "questions": [
        {"kind": "choice", "id": "operation", "prompt": "次に行う操作の種別を選ぶ", "options": ["click", "back"]},
        {"kind": "choice", "id": "target", "prompt": "操作する要素の番号を選ぶ", "options": ["1"]},
        {"kind": "noul", "id": "error_shown", "prompt": "画面にエラーの表示があるか"}
    ]});
    let second = json!({"version": 1, "seq": 2, "screen": "", "questions": []});
    [first, second]
}
