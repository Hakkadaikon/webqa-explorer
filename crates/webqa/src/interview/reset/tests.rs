//! Tests of running the reset command with the scripts in
//! `tests/fixtures/reset/`.

use std::env::var_os;
use std::future;
use std::num::NonZeroU64;
use std::time::Duration;

use webqa_core::ports::Pending;
use webqa_core::redact::Rule;

use super::*;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reset/");

/// Never lets the timeout pass, and keeps every deadline it is asked about.
struct NeverClock {
    now: Duration,
    asked: Vec<Duration>,
}

impl NeverClock {
    const fn at(secs: u64) -> Self {
        Self {
            now: Duration::from_secs(secs),
            asked: Vec::new(),
        }
    }
}

impl Clock for NeverClock {
    fn elapsed(&self) -> Duration {
        self.now
    }

    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        self.asked.push(at);
        Box::pin(future::pending())
    }
}

/// Lets the timeout pass at once.
struct InstantClock;

impl Clock for InstantClock {
    fn elapsed(&self) -> Duration {
        Duration::ZERO
    }

    fn sleep_until(&mut self, _at: Duration) -> Pending<'_, ()> {
        Box::pin(future::ready(()))
    }
}

fn reset(script: &str, args: &[&str], timeout_secs: u64) -> Reset {
    Reset {
        program: format!("{FIXTURES}{script}"),
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
        timeout_secs: NonZeroU64::new(timeout_secs).unwrap(),
    }
}

/// Runs `script` with a timeout of 5, no environment, nothing withheld, no
/// rules, and a clock that never lets the timeout pass.
async fn ran(script: &str, args: &[&str]) -> Ran {
    let mut clock = NeverClock::at(0);
    run(
        &reset(script, args, 5),
        &[],
        &[],
        &Redactor::default(),
        &mut clock,
    )
    .await
}

fn silent() -> Output {
    let plain = Redactor::default();
    Output {
        stdout: plain.text(""),
        stderr: plain.text(""),
    }
}

fn failed(error: ResetError) -> Ran {
    Ran {
        result: Err(error),
        output: silent(),
    }
}

/// The scripts that exit by themselves, their argument, and the result.
const EXITS: [(&str, &str, Result<(), ResetError>); 5] = [
    ("exit.sh", "0", Ok(())),
    ("exit.sh", "3", Err(ResetError::Exit(3))),
    ("exit.sh", "1", Err(ResetError::Exit(1))),
    ("exit.sh", "255", Err(ResetError::Exit(255))),
    ("signal.sh", "", Err(ResetError::Signaled)),
];

/// Every error and its message.
const MESSAGES: [(ResetError, &str); 4] = [
    (ResetError::Launch, "初期化のコマンドを起動できない"),
    (
        ResetError::Exit(3),
        "初期化のコマンドが終了コード 3 で終わった",
    ),
    (ResetError::Signaled, "初期化のコマンドがシグナルで止まった"),
    (
        ResetError::TimedOut,
        "初期化のコマンドが待ち時間内に終わらない",
    ),
];

/// Runs `sleep.sh`, which needs `PATH`, with a clock that lets the timeout
/// pass at once.
async fn slept() -> Ran {
    let path = [(OsString::from("PATH"), var_os("PATH").unwrap())];
    let slow = reset("sleep.sh", &[], 5);
    run(&slow, &path, &[], &Redactor::default(), &mut InstantClock).await
}

#[tokio::test]
async fn nonzero_or_timeout_is_failure() {
    for (script, arg, result) in EXITS {
        let expected = Ran {
            result,
            output: silent(),
        };
        assert_eq!(ran(script, &[arg]).await, expected, "{script} {arg}");
    }
    assert_eq!(slept().await, failed(ResetError::TimedOut), "sleep.sh");
    for (error, message) in MESSAGES {
        assert_eq!(error.to_string(), message, "{error:?}");
    }
}

#[tokio::test]
async fn unlaunchable_command_is_failure() {
    for script in ["no-such-command", "not-executable.sh"] {
        let mut clock = NeverClock::at(0);
        let reset = reset(script, &[], 5);
        let plain = Redactor::default();
        let ran = run(&reset, &[], &[], &plain, &mut clock).await;
        let seen = (ran, clock.asked);
        assert_eq!(seen, (failed(ResetError::Launch), vec![]), "{script}");
    }
}

#[tokio::test]
async fn deadline_is_start_plus_timeout() {
    let cases = [
        (100, 5, Duration::from_secs(105)),
        (100, u64::MAX, Duration::MAX),
        (0, 1, Duration::from_secs(1)),
    ];
    for (now, timeout, deadline) in cases {
        let mut clock = NeverClock::at(now);
        let exit = reset("exit.sh", &["0"], timeout);
        let plain = Redactor::default();
        let ran = run(&exit, &[], &[], &plain, &mut clock).await;
        let seen = (ran.result, clock.asked);
        let expected = (Ok(()), vec![deadline]);
        assert_eq!(seen, expected, "now {now}, timeout {timeout}");
    }
}

/// Masks mail addresses.
fn mail() -> Redactor {
    let rule = Rule::pattern("メールアドレス", r"[\w.+-]+@[\w-]+\.[\w.]+");
    Redactor::new(&[rule]).unwrap()
}

/// Runs `script` with `args` through [`mail`].
async fn masked(script: &str, args: &[&str]) -> Ran {
    let mut clock = NeverClock::at(0);
    let reset = reset(script, args, 5);
    run(&reset, &[], &[], &mail(), &mut clock).await
}

#[tokio::test]
async fn logged_output_is_redacted() {
    let plain = Redactor::default();
    let written = Output {
        stdout: plain.text("reset for [[メールアドレス]]\n"),
        stderr: plain.text("warn [[メールアドレス]]\n"),
    };
    let lines = ["reset for test@example.test", "warn test@example.test"];
    let codes = [(None, Ok(())), (Some("4"), Err(ResetError::Exit(4)))];
    for (code, result) in codes {
        let args: Vec<&str> = lines.into_iter().chain(code).collect();
        let ran = masked("echo.sh", &args).await;
        let shown = format!("{ran:?}");
        assert!(!shown.contains("test@example.test"), "{shown}");
        let output = written.clone();
        assert_eq!(ran, Ran { result, output }, "exit code {code:?}");
    }
    let bytes = masked("bytes.sh", &[]).await.output;
    let seen = (bytes.stdout.into_inner(), bytes.stderr.into_inner());
    assert_eq!(seen, ("a\u{FFFD}b".to_owned(), String::new()), "bytes.sh");
}

#[tokio::test]
async fn errors_carry_no_command() {
    let cases = [
        ("exit.sh", "exit.sh", ResetError::Exit(5)),
        ("not-executable.sh", "not-executable", ResetError::Launch),
    ];
    for (script, name, error) in cases {
        let ran = ran(script, &["5", "secret-arg-123"]).await;
        let shown = format!("{ran:?} {error}");
        let leaks = (shown.contains("secret-arg-123"), shown.contains(name));
        let expected = (Err(error), (false, false));
        assert_eq!((ran.result, leaks), expected, "{shown}");
    }
}
