//! Contract tests of the reset command: real child processes started from
//! the scripts in `tests/fixtures/reset/`.

use std::env::var_os;
use std::ffi::OsString;
use std::fs;
use std::future;
use std::io;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use tokio::task::yield_now;
use webqa::interview::reset::{Output, Ran, ResetError, run};
use webqa_core::config::interview::Reset;
use webqa_core::ports::{Clock, Pending};
use webqa_core::redact::Redactor;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reset/");

/// Environment variables, as `run` takes them.
type Vars = [(OsString, OsString)];

/// Never lets the timeout pass.
struct NeverClock;

impl Clock for NeverClock {
    fn elapsed(&self) -> Duration {
        Duration::ZERO
    }

    fn sleep_until(&mut self, _at: Duration) -> Pending<'_, ()> {
        Box::pin(future::pending())
    }
}

/// Lets the timeout pass once something is written to `file`.
struct GateClock {
    file: PathBuf,
}

impl Clock for GateClock {
    fn elapsed(&self) -> Duration {
        Duration::ZERO
    }

    fn sleep_until(&mut self, _at: Duration) -> Pending<'_, ()> {
        Box::pin(wait_for(self.file.clone()))
    }
}

/// Yields until something is written to `file`.
async fn wait_for(file: PathBuf) {
    while !written(&file) {
        yield_now().await;
    }
}

/// Whether `file` has something in it.
fn written(file: &Path) -> bool {
    fs::metadata(file).is_ok_and(|meta| meta.len() > 0)
}

/// A fresh directory for one test.
fn scratch(test: &str) -> io::Result<PathBuf> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("reset");
    let dir = dir.join(test);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn reset(script: &str, args: &[&str]) -> Reset {
    Reset {
        program: format!("{FIXTURES}{script}"),
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
        // Five seconds.
        timeout_secs: NonZeroU64::MIN.saturating_add(4),
    }
}

fn var(name: &str, value: impl Into<OsString>) -> (OsString, OsString) {
    (OsString::from(name), value.into())
}

/// Runs `record.sh` with a clock that never lets the timeout pass.
async fn record(args: &[&str], env: &Vars, withheld: &[&str]) -> Result<(), ResetError> {
    let redactor = Redactor::default();
    let reset = reset("record.sh", args);
    let mut clock = NeverClock;
    run(&reset, env, withheld, &redactor, &mut clock)
        .await
        .result
}

/// The arguments `record.sh` wrote to `file`, each followed by a NUL.
fn recorded(file: &Path) -> io::Result<Vec<String>> {
    let bytes = fs::read(file)?;
    let text = String::from_utf8_lossy(&bytes);
    let mut args: Vec<String> = text.split('\0').map(str::to_owned).collect();
    assert_eq!(args.pop(), Some(String::new()), "{text:?} ends with NUL");
    Ok(args)
}

/// Runs `record.sh` with no arguments, and returns its result and the bytes
/// it wrote for them.
async fn record_none(dir: &Path) -> io::Result<(Result<(), ResetError>, Vec<u8>)> {
    let env = [var("RESET_OUT", dir.join("none"))];
    let result = record(&[], &env, &[]).await;
    Ok((result, fs::read(dir.join("none.args"))?))
}

/// Whether the process `pid` is still there.
fn alive(pid: &str) -> io::Result<bool> {
    let status = Command::new("/bin/sh")
        .args(["-c", "kill -0 \"$1\" 2>/dev/null", "sh", pid])
        .status()?;
    Ok(status.success())
}

#[tokio::test]
async fn args_passed_verbatim_without_shell() {
    let dir = scratch("args").unwrap();
    let injected = dir.join("INJECTED");
    let first = format!("a; touch {}", injected.display());
    let args = [
        first.as_str(),
        "$HOME",
        "*",
        "",
        "two words",
        "--x=\"q\"",
        "ねこ",
    ];
    let env = [var("RESET_OUT", dir.join("out"))];
    let result = record(&args, &env, &[]).await;
    let seen = recorded(&dir.join("out.args")).unwrap();
    let expected = args.map(str::to_owned).to_vec();
    assert_eq!((result, seen), (Ok(()), expected), "args");
    assert!(!injected.exists(), "{} was made", injected.display());
    let none = record_none(&dir).await.unwrap();
    assert_eq!(none, (Ok(()), vec![]), "no args");
}

#[tokio::test]
async fn api_key_env_not_inherited() {
    let dir = scratch("env").unwrap();
    let env = [
        var("RESET_OUT", dir.join("out")),
        var("JEV_API_KEY", "value-of-JEV_API_KEY"),
        var("AUTH_HEADER", "Bearer value-of-JEV_API_KEY"),
        var("KEEP", "kept"),
    ];
    let result = record(&[], &env, &["value-of-JEV_API_KEY"]).await;
    assert_eq!(result, Ok(()), "record.sh");
    let exported = fs::read_to_string(dir.join("out.env")).unwrap();
    let words = [
        "KEEP",
        "kept",
        "JEV_API_KEY",
        "AUTH_HEADER",
        "value-of-",
        "CARGO_MANIFEST_DIR",
    ];
    let found = words.map(|word| exported.contains(word));
    let expected = [true, true, false, false, false, false];
    assert_eq!(found, expected, "{words:?} in {exported}");
}

#[tokio::test]
async fn timed_out_command_is_stopped() {
    let dir = scratch("timeout").unwrap();
    let pid_file = dir.join("pid");
    let env = [
        var("RESET_OUT", &pid_file),
        var("PATH", var_os("PATH").unwrap()),
    ];
    let mut clock = GateClock {
        file: pid_file.clone(),
    };
    let plain = Redactor::default();
    let reset = reset("hang.sh", &[]);
    let ran = run(&reset, &env, &[], &plain, &mut clock).await;
    let silent = Output {
        stdout: plain.text(""),
        stderr: plain.text(""),
    };
    let expected = Ran {
        result: Err(ResetError::TimedOut),
        output: silent,
    };
    assert_eq!(ran, expected, "hang.sh");
    let pid = fs::read_to_string(&pid_file).unwrap();
    assert!(!pid.is_empty(), "hang.sh wrote no pid");
    assert!(!alive(&pid).unwrap(), "process {pid} is still alive");
}
