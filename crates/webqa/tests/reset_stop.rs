//! Contract tests of how the reset command is waited for and stopped: real
//! child processes started from the scripts in `tests/fixtures/reset/`.

use std::env::var_os;
use std::ffi::OsString;
use std::fs;
use std::future;
use std::io;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::task::yield_now;
use webqa::interview::reset::{Output, Ran, ResetError, run};
use webqa_core::config::interview::Reset;
use webqa_core::ports::{Clock, Pending};
use webqa_core::redact::Redactor;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reset/");

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

/// Runs `script` with `PATH`, and lets the timeout pass once it has written
/// to `<dir>/pid`. Returns how it went and what it wrote there.
async fn time_out(dir: &Path, script: &str) -> io::Result<(Ran, String)> {
    let pid_file = dir.join("pid");
    let env = [var("RESET_OUT", &pid_file), path()];
    let mut clock = GateClock {
        file: pid_file.clone(),
    };
    let reset = reset(script, &[]);
    let ran = run(&reset, &env, &[], &Redactor::default(), &mut clock).await;
    Ok((ran, fs::read_to_string(&pid_file)?))
}

fn path() -> (OsString, OsString) {
    var("PATH", var_os("PATH").unwrap_or_default())
}

/// The run that timed out.
fn timed_out() -> Ran {
    let plain = Redactor::default();
    Ran {
        result: Err(ResetError::TimedOut),
        output: Output {
            stdout: plain.text(""),
            stderr: plain.text(""),
        },
    }
}

/// Whether the process `pid` is there at all, even as a zombie.
fn exists(pid: &str) -> bool {
    Path::new("/proc").join(pid).exists()
}

/// Whether the process `pid` is still running: there, and not a zombie.
fn running(pid: &str) -> bool {
    let stat = fs::read_to_string(Path::new("/proc").join(pid).join("stat"));
    stat.is_ok_and(|stat| {
        stat.rsplit_once(") ")
            .is_some_and(|(_, rest)| !rest.starts_with('Z'))
    })
}

/// Whether the process `pid`, which has been sent SIGKILL, stops running
/// soon.
fn stops(pid: &str) -> bool {
    (0..1_000_000).any(|_| !running(pid))
}

#[tokio::test]
async fn timed_out_command_is_stopped() {
    let dir = scratch("timeout").unwrap();
    let (ran, pid) = time_out(&dir, "hang.sh").await.unwrap();
    assert_eq!(ran, timed_out(), "hang.sh");
    assert!(!pid.is_empty(), "hang.sh wrote no pid");
    assert!(!exists(&pid), "process {pid} is left, maybe as a zombie");
}

#[tokio::test]
async fn timed_out_group_is_stopped() {
    let dir = scratch("group").unwrap();
    let (ran, pids) = time_out(&dir, "spawn.sh").await.unwrap();
    let (pid, grandchild) = pids.split_once(' ').unwrap();
    assert_eq!(ran, timed_out(), "spawn.sh");
    assert!(!exists(pid), "process {pid} is left, maybe as a zombie");
    assert!(
        stops(grandchild),
        "grandchild {grandchild} is still running"
    );
}

#[tokio::test]
async fn full_pipes_do_not_stall() {
    let reset = reset("flood.sh", &[]);
    let plain = Redactor::default();
    let ran = run(&reset, &[path()], &[], &plain, &mut NeverClock).await;
    let stdout = ran.output.stdout.as_inner().len();
    let stderr = ran.output.stderr.as_inner().len();
    assert_eq!(
        (ran.result, stdout, stderr),
        (Ok(()), 1 << 20, 1 << 20),
        "flood.sh"
    );
}
