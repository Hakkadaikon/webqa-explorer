//! Tests of following a run's event log.

use std::env::temp_dir;
use std::fs::{self, OpenOptions};
use std::future::{pending, ready};
use std::io::{ErrorKind, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::id;
use std::time::Duration;

use webqa_core::ports::{Clock, Pending};

use super::*;

const STARTED: &str = r#"{"seq":0,"elapsed_ms":0,"kind":"run_started","schema":1,"mode":"explore","started_at":"2026-10-18T02:00:00Z"}"#;
const DECIDED: &str =
    r#"{"seq":1,"elapsed_ms":1200,"step":1,"kind":"decided","op":"click","target":"保存","p":0.9}"#;
const FINISHED: &str =
    r#"{"seq":9,"elapsed_ms":60000,"kind":"run_finished","reason":"goal_achieved"}"#;

/// A new empty directory for the test `name`, removed when dropped.
struct Workdir(PathBuf);

impl Deref for Workdir {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for Workdir {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.0));
    }
}

fn workdir(name: &str) -> Workdir {
    let dir = temp_dir().join(format!("webqa-watch-{name}-{}", id()));
    drop(fs::remove_dir_all(&dir));
    fs::create_dir_all(&dir).unwrap();
    Workdir(dir)
}

fn append(path: &Path, text: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();
    file.write_all(text.as_bytes()).unwrap();
}

/// Waits without delay, and appends `FINISHED` to the log on the wait
/// numbered `finish_on`, from 1.
struct FakeClock {
    log: PathBuf,
    finish_on: usize,
    sleeps: Vec<Duration>,
}

impl Clock for FakeClock {
    fn elapsed(&self) -> Duration {
        Duration::ZERO
    }

    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        self.sleeps.push(at);
        if self.sleeps.len() == self.finish_on {
            append(&self.log, &format!("{FINISHED}\n"));
        }
        // The fourth wait never ends, so a loop that does not stop ends
        // with the wrong waits instead of spinning.
        let waits: [Pending<'_, ()>; 4] = [
            Box::pin(ready(())),
            Box::pin(ready(())),
            Box::pin(ready(())),
            Box::pin(pending()),
        ];
        let turn = self.sleeps.len().min(4) - 1;
        waits.into_iter().nth(turn).expect("one of four waits")
    }
}

/// Interrupts as soon as the work waits.
struct AtOnce;

impl Interrupt for AtOnce {
    fn wait(&mut self) -> Pending<'_, ()> {
        Box::pin(async {})
    }
}

/// The text of terminal output, without the escape sequences.
fn plain(bytes: &[u8]) -> String {
    let step = |escaped: &mut bool, c: char| {
        let inside = *escaped || c == '\x1b';
        *escaped = inside && !c.is_ascii_alphabetic();
        Some((!inside).then_some(c))
    };
    let text = String::from_utf8_lossy(bytes);
    text.chars().scan(false, step).flatten().collect()
}

#[test]
fn appended_reads_after_offset() {
    let dir = workdir("appended");
    let path = dir.join("events.jsonl");
    assert_eq!(appended(&path, 0).unwrap(), (vec![], 0));
    append(&path, "0123456789");
    assert_eq!(appended(&path, 0).unwrap(), (b"0123456789".to_vec(), 10));
    append(&path, "abcde");
    assert_eq!(appended(&path, 10).unwrap(), (b"abcde".to_vec(), 15));
    fs::write(&path, "xyz").unwrap();
    assert_eq!(appended(&path, 15).unwrap(), (vec![], 15));
}

#[test]
fn appended_reports_a_log_that_cannot_be_read() {
    let dir = workdir("unreadable");
    let read = appended(&dir, 0);
    assert_eq!(read.map_err(|e| e.kind()), Err(ErrorKind::IsADirectory));
}

#[tokio::test]
async fn watch_draws_and_waits_after_finish() {
    let dir = workdir("finish");
    let log = dir.join("events.jsonl");
    append(&log, &format!("{STARTED}\n{DECIDED}\n"));
    let mut clock = FakeClock {
        log,
        finish_on: 2,
        sleeps: vec![],
    };
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = watch(&dir, &mut clock, &mut AtOnce, &mut out, &mut err).await;
    assert_eq!(code, 0);
    assert_eq!(
        clock.sleeps,
        [Duration::from_millis(200), Duration::from_millis(400)],
        "it reads every 200 ms, and stops reading once the run finished"
    );
    let text = plain(&out);
    assert!(text.contains("保存"), "{text:?}");
    assert!(text.contains("goal_achieved"), "{text:?}");
}

#[tokio::test]
async fn missing_dir_exits_two() {
    let base = workdir("missing");
    let dir = base.join("nothing");
    let mut clock = FakeClock {
        log: dir.join("events.jsonl"),
        finish_on: 0,
        sleeps: vec![],
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(
        watch(&dir, &mut clock, &mut AtOnce, &mut out, &mut err).await,
        2
    );
    assert_eq!((out, clock.sleeps), (vec![], vec![]));
    assert_eq!(err, "実行のディレクトリが無い\n".as_bytes());
}
