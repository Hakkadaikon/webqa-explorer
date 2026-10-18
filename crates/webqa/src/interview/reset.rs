//! Runs the command that resets the service before each trial.
//!
//! The command is started without a shell, gets only the environment it is
//! given minus the judge's API key, and is stopped when its timeout passes on
//! the injected clock. Its output comes back only redacted.

mod env;
#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::io;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, empty};
use tokio::process::{Child, Command};

use webqa_core::config::interview::Reset;
use webqa_core::ports::Clock;
use webqa_core::redact::{Redacted, Redactor};

/// Why the reset command failed. Carries neither the program nor its
/// arguments nor its output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ResetError {
    /// The command could not be started, or waiting on it failed.
    #[error("初期化のコマンドを起動できない")]
    Launch,
    /// The command exited with a code other than 0.
    #[error("初期化のコマンドが終了コード {0} で終わった")]
    Exit(i32),
    /// The command was ended by a signal.
    #[error("初期化のコマンドがシグナルで止まった")]
    Signaled,
    /// The command did not finish within its timeout. It has been stopped.
    #[error("初期化のコマンドが待ち時間内に終わらない")]
    TimedOut,
}

/// What the command wrote, redacted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// The standard output.
    pub stdout: Redacted<String>,
    /// The standard error.
    pub stderr: Redacted<String>,
}

/// How a run of the reset command went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ran {
    /// `Ok` when the command exited with 0 in time.
    pub result: Result<(), ResetError>,
    /// What it wrote. Empty when it could not start or timed out.
    pub output: Output,
}

/// Runs the reset command once, without a shell.
///
/// The command gets `env` and nothing else from this process, minus every
/// variable whose value contains one of `withheld`. Its output passes
/// through `redactor`. It is stopped when `reset.timeout_secs` pass on
/// `clock`.
pub async fn run(
    reset: &Reset,
    env: &[(OsString, OsString)],
    withheld: &[&str],
    redactor: &Redactor,
    clock: &mut dyn Clock,
) -> Ran {
    let deadline = deadline(clock.elapsed(), reset);
    match command(reset, env, withheld).spawn() {
        Ok(child) => supervise(child, deadline, redactor, clock).await,
        Err(_) => silent(ResetError::Launch, redactor),
    }
}

/// When the command must have finished, measured from the start of the run.
const fn deadline(start: Duration, reset: &Reset) -> Duration {
    start.saturating_add(Duration::from_secs(reset.timeout_secs.get()))
}

/// The command, ready to start without a shell.
fn command(reset: &Reset, env: &[(OsString, OsString)], withheld: &[&str]) -> Command {
    let mut command = Command::new(&reset.program);
    command
        .args(&reset.args)
        .env_clear()
        .envs(env::passed(env, withheld))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    command
}

/// Waits for the started command until `deadline`, and stops it if it has
/// not finished by then.
async fn supervise(
    mut child: Child,
    deadline: Duration,
    redactor: &Redactor,
    clock: &mut dyn Clock,
) -> Ran {
    let finished = tokio::select! {
        biased;
        finished = finish(&mut child) => Some(finished),
        () = clock.sleep_until(deadline) => None,
    };
    match finished {
        Some(finished) => finished.ran(redactor),
        None => stop(child, redactor).await,
    }
}

/// Stops the command that did not finish in time, and waits until it has.
async fn stop(mut child: Child, redactor: &Redactor) -> Ran {
    let _ = child.kill().await;
    silent(ResetError::TimedOut, redactor)
}

/// The failure with no output.
fn silent(error: ResetError, redactor: &Redactor) -> Ran {
    Ran {
        result: Err(error),
        output: Output {
            stdout: redactor.text(""),
            stderr: redactor.text(""),
        },
    }
}

/// How the command ended and what it wrote.
struct Finished {
    status: io::Result<ExitStatus>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Finished {
    fn ran(self, redactor: &Redactor) -> Ran {
        let text = |bytes: &[u8]| redactor.text(&String::from_utf8_lossy(bytes));
        Ran {
            result: self.status.map_or(Err(ResetError::Launch), exit_result),
            output: Output {
                stdout: text(&self.stdout),
                stderr: text(&self.stderr),
            },
        }
    }
}

/// Reads both pipes to the end while waiting for the command to exit, so a
/// full pipe never keeps it from finishing.
async fn finish(child: &mut Child) -> Finished {
    let stdout = reader(child.stdout.take());
    let stderr = reader(child.stderr.take());
    let (status, stdout, stderr) = tokio::join!(child.wait(), read_all(stdout), read_all(stderr));
    Finished {
        status,
        stdout,
        stderr,
    }
}

/// A pipe of the command, read through one type.
type Pipe = Box<dyn AsyncRead + Unpin + Send>;

/// The reader of `pipe`, or one with nothing in it when there is none.
fn reader(pipe: Option<impl AsyncRead + Unpin + Send + 'static>) -> Pipe {
    let nothing: Pipe = Box::new(empty());
    pipe.map_or(nothing, |pipe| Box::new(pipe))
}

/// Everything that could be read from `pipe`. A failed read ends it.
async fn read_all(mut pipe: Pipe) -> Vec<u8> {
    let mut bytes = Vec::new();
    let _ = pipe.read_to_end(&mut bytes).await;
    bytes
}

/// The result for how the command exited.
fn exit_result(status: ExitStatus) -> Result<(), ResetError> {
    if status.success() {
        return Ok(());
    }
    Err(status.code().map_or(ResetError::Signaled, ResetError::Exit))
}
