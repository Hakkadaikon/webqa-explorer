//! A judge that talks to an external bridge command, one JSON line each way.
//!
//! The command starts once, at the start of a run. Each question set is one
//! request line on its standard input, and the answer is one line on its
//! standard output. Both carry [`PROTOCOL_VERSION`] and a `seq` that grows
//! with each request, so a late answer to a request that timed out is
//! skipped.

mod wire;

use std::io;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Split};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::task::yield_now;
use webqa_core::judge::{Answers, JudgeError, Question};
use webqa_core::ports::{Clock, Judge, Pending};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;

use crate::table::StartError;

/// The version of the bridge format written in every line.
pub const PROTOCOL_VERSION: u32 = 1;

/// The environment variable that hands the API key to the command.
const KEY_ENV: &str = "WEBQA_JUDGE_API_KEY";

/// How to start the bridge. Holds the key only until the command starts;
/// no `Debug`.
pub struct Bridge<'a> {
    /// Program and its arguments. Run directly, never through a shell.
    pub command: &'a [String],
    /// API key handed to the command, if the configuration names one.
    pub api_key: Option<&'a str>,
    /// How long to wait for each answer.
    pub timeout: Duration,
}

/// The judge behind a running bridge command. Holds no key.
pub struct CommandJudge {
    /// Killed when the judge is dropped.
    _child: Child,
    stdin: ChildStdin,
    stdout: Split<BufReader<ChildStdout>>,
    clock: Box<dyn Clock>,
    timeout: Duration,
    seq: u64,
}

impl CommandJudge {
    /// Starts the bridge command.
    ///
    /// # Errors
    ///
    /// [`StartError::NoCommand`] when `bridge.command` is empty, and
    /// [`StartError::Spawn`] when the command does not start.
    pub fn spawn(bridge: &Bridge<'_>, clock: Box<dyn Clock>) -> Result<Self, StartError> {
        let (program, args) = bridge.command.split_first().ok_or(StartError::NoCommand)?;
        let mut child = command(program, args, bridge.api_key)
            .spawn()
            .map_err(spawn_error)?;
        let pipes = child.stdin.take().zip(child.stdout.take());
        pipes
            .map(|(stdin, stdout)| Self {
                _child: child,
                stdin,
                stdout: BufReader::new(stdout).split(b'\n'),
                clock,
                timeout: bridge.timeout,
                seq: 0,
            })
            .ok_or(StartError::Spawn)
    }

    /// Sends the request and waits for its answer, or for the time to run
    /// out.
    async fn exchange(
        &mut self,
        screen: &Redacted<ScreenText>,
        questions: &[Question],
    ) -> Result<Answers, JudgeError> {
        self.seq += 1;
        let line = wire::request(self.seq, screen.as_inner().as_str(), questions);
        let deadline = self.clock.elapsed() + self.timeout;
        let Self {
            stdin,
            stdout,
            clock,
            seq,
            ..
        } = self;
        tokio::select! {
            biased;
            answers = talk(stdin, stdout, &line, *seq) => answers,
            () = expire(clock.as_mut(), deadline) => Err(JudgeError::Timeout),
        }
    }
}

impl Judge for CommandJudge {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        Box::pin(self.exchange(screen, questions))
    }
}

/// Waits until `deadline`, after one turn of the runtime's I/O driver.
///
/// A fresh pipe is writable only once the driver has seen it, so without
/// the turn a deadline that has already passed would win before the request
/// is written, and the late answer could never be skipped.
async fn expire(clock: &mut dyn Clock, deadline: Duration) {
    yield_now().await;
    clock.sleep_until(deadline).await;
}

/// The command with pipes for its input and output, its errors thrown away
/// (they may hold page strings or secrets), and the key in its environment
/// only when there is one.
fn command(program: &str, args: &[String], api_key: Option<&str>) -> Command {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    match api_key {
        Some(key) => command.env(KEY_ENV, key),
        None => command.env_remove(KEY_ENV),
    };
    command
}

fn spawn_error(_: io::Error) -> StartError {
    StartError::Spawn
}

/// Writes the request and reads lines until the answer to `seq`.
///
/// A failed write is not reported by itself. It means the command has
/// ended (or closed its input), and whether the write or the read notices
/// first depends on timing. The read reports the end of the output as
/// [`JudgeError::Bridge`]; a command that closed its input but still runs
/// gives no answer, which ends as a timeout.
async fn talk(
    stdin: &mut ChildStdin,
    stdout: &mut Split<BufReader<ChildStdout>>,
    line: &str,
    seq: u64,
) -> Result<Answers, JudgeError> {
    stdin.write_all(line.as_bytes()).await.ok();
    stdin.flush().await.ok();
    receive(stdout, seq).await
}

/// Reads lines until the answer to `seq`, skipping late answers.
async fn receive(
    stdout: &mut Split<BufReader<ChildStdout>>,
    seq: u64,
) -> Result<Answers, JudgeError> {
    loop {
        if let Some(answers) = wire::read(&next_line(stdout).await?, seq)? {
            return Ok(answers);
        }
    }
}

/// The next line as bytes, or [`JudgeError::Bridge`] once the command has
/// ended or its output cannot be read. A line that is not UTF-8 is left to
/// [`wire::read`], which finds it malformed.
async fn next_line(stdout: &mut Split<BufReader<ChildStdout>>) -> Result<Vec<u8>, JudgeError> {
    stdout
        .next_segment()
        .await
        .ok()
        .flatten()
        .ok_or(JudgeError::Bridge)
}
