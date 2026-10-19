//! Tests of `webqa watch`, the executable.

use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind, Read, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{ChildStdout, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

const STARTED: &str = r#"{"seq":0,"elapsed_ms":0,"kind":"run_started","schema":1,"mode":"explore","started_at":"2026-10-18T02:00:00Z"}"#;
const DECIDED: &str =
    r#"{"seq":1,"elapsed_ms":1200,"step":1,"kind":"decided","op":"click","target":"保存","p":0.9}"#;
const FINISHED: &str =
    r#"{"seq":9,"elapsed_ms":60000,"kind":"run_finished","reason":"goal_achieved"}"#;
const LIMIT: Duration = Duration::from_secs(10);

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

fn append(dir: &Path, text: &str) -> io::Result<()> {
    let log = dir.join("events.jsonl");
    let mut file = OpenOptions::new().create(true).append(true).open(log)?;
    file.write_all(text.as_bytes())
}

/// `webqa watch dir` with its standard output read as it comes.
struct Watching {
    pid: u32,
    ended: Receiver<io::Result<ExitStatus>>,
    seen: Vec<u8>,
    chunks: Receiver<Vec<u8>>,
}

/// Sends what `stdout` yields, chunk by chunk, until it closes.
fn forward(mut stdout: ChildStdout) -> Receiver<Vec<u8>> {
    let (send, chunks) = mpsc::channel();
    thread::spawn(move || {
        let mut buffer = [0; 4096];
        while let Ok(n @ 1..) = stdout.read(&mut buffer) {
            drop(send.send(buffer.get(..n).unwrap_or_default().to_vec()));
        }
    });
    chunks
}

fn watching(dir: &Path) -> io::Result<Watching> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_webqa"))
        .arg("watch")
        .arg(dir)
        // Its own process group, so the test can signal it as a terminal
        // does for Ctrl-C.
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child.stdout.take().ok_or(ErrorKind::BrokenPipe)?;
    let pid = child.id();
    let (send, ended) = mpsc::channel();
    thread::spawn(move || drop(send.send(child.wait())));
    Ok(Watching {
        pid,
        ended,
        seen: vec![],
        chunks: forward(stdout),
    })
}

impl Drop for Watching {
    /// Stops the whole group, so that a test that ends early leaves nothing running.
    fn drop(&mut self) {
        let group = format!("-{}", self.pid);
        drop(Command::new("kill").args(["-KILL", "--", &group]).status());
    }
}

impl Watching {
    /// Reads the output until it shows `text`; each chunk may take [`LIMIT`].
    fn until_shown(&mut self, text: &str) -> io::Result<()> {
        while !plain(&self.seen).contains(text) {
            let chunk = self.chunks.recv_timeout(LIMIT);
            self.seen.extend(chunk.map_err(|_| ErrorKind::TimedOut)?);
        }
        Ok(())
    }

    /// Sends Ctrl-C to the group, and waits for the exit code.
    fn interrupt(&self) -> io::Result<Option<i32>> {
        let group = format!("-{}", self.pid);
        let status = Command::new("kill").args(["-INT", "--", &group]).status()?;
        assert!(status.success(), "kill sent the signal");
        let ended = self.ended.recv_timeout(LIMIT);
        Ok(ended.map_err(|_| ErrorKind::TimedOut)??.code())
    }
}

/// A new empty directory for the test `name`.
fn fresh(name: &str) -> io::Result<PathBuf> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    drop(fs::remove_dir_all(&dir));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Appends `line` to the log in `dir`, and waits for `text` on the screen.
fn feed(watching: &mut Watching, dir: &Path, line: &str, text: &str) -> io::Result<()> {
    append(dir, &format!("{line}\n"))?;
    watching.until_shown(text)
}

#[test]
fn watch_follows_a_live_log() -> io::Result<()> {
    let dir = fresh("watch_live")?;
    let mut watching = watching(&dir)?;
    feed(
        &mut watching,
        &dir,
        &format!("{STARTED}\n{DECIDED}"),
        "保存",
    )?;
    feed(&mut watching, &dir, FINISHED, "goal_achieved")?;
    assert_eq!(watching.interrupt()?, Some(0));
    Ok(())
}

#[test]
fn watching_a_missing_directory_exits_two() -> io::Result<()> {
    let missing = Path::new(env!("CARGO_TARGET_TMPDIR")).join("watch_missing");
    drop(fs::remove_dir_all(&missing));
    let status = Command::new(env!("CARGO_BIN_EXE_webqa"))
        .arg("watch")
        .arg(&missing)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    assert_eq!(status.code(), Some(2));
    Ok(())
}
