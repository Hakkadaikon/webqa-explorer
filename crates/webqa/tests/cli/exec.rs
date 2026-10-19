//! Starts the `webqa` executable and waits for it.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use serde_json::Value;

/// The longest any wait lasts.
pub const LIMIT: Duration = Duration::from_secs(60);

/// A new empty working directory for the test `name`.
pub fn workdir(name: &str) -> io::Result<PathBuf> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("cli")
        .join(name);
    drop(fs::remove_dir_all(&dir));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// A started `webqa`.
pub struct Running {
    /// The process id, for signals.
    pub pid: u32,
    done: Receiver<io::Result<Output>>,
}

/// Starts `webqa` with `args` in `dir`.
pub fn start<S: AsRef<OsStr>>(dir: &Path, args: &[S]) -> io::Result<Running> {
    let child = Command::new(env!("CARGO_BIN_EXE_webqa"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let pid = child.id();
    let (send, done) = mpsc::channel();
    thread::spawn(move || drop(send.send(child.wait_with_output())));
    Ok(Running { pid, done })
}

impl Running {
    /// Waits `limit` for the end; `None` when it did not end.
    pub fn wait(&self, limit: Duration) -> io::Result<Option<Output>> {
        self.done
            .recv_timeout(limit)
            .ok()
            .map_or(Ok(None), |ended| ended.map(Some))
    }

    /// Waits for the end, up to [`LIMIT`].
    pub fn finish(&self) -> io::Result<Output> {
        let ended = self.wait(LIMIT)?;
        ended.ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "webqa did not end"))
    }
}

/// Runs `webqa` with `args` in `dir` to its end.
pub fn run<S: AsRef<OsStr>>(dir: &Path, args: &[S]) -> io::Result<Output> {
    start(dir, args)?.finish()
}

/// The exit code of `output`.
pub fn code(output: &Output) -> Option<i32> {
    output.status.code()
}

/// The standard error of `output`.
pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The names in `dir`, sorted. A directory that cannot be read has none.
pub fn names(dir: &Path) -> Vec<String> {
    let entries = fs::read_dir(dir).map(Iterator::collect::<Vec<_>>);
    let mut names: Vec<String> = entries
        .unwrap_or_default()
        .iter()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// The directory of the latest run under `out`.
pub fn latest_run(out: &Path) -> io::Result<PathBuf> {
    let runs = out.join("runs");
    let latest = names(&runs).pop();
    latest
        .map(|name| runs.join(name))
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
}

/// `report.json` of `run`.
pub fn report(run: &Path) -> io::Result<Value> {
    let text = fs::read_to_string(run.join("report.json"))?;
    serde_json::from_str(&text).map_err(io::Error::other)
}

/// The lines of `events.jsonl` of `run`.
pub fn events(run: &Path) -> io::Result<Vec<Value>> {
    let text = fs::read_to_string(run.join("events.jsonl"))?;
    let lines = text.lines().map(serde_json::from_str);
    lines.collect::<Result<_, _>>().map_err(io::Error::other)
}
