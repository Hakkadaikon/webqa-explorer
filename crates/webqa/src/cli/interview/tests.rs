//! Tests of the arguments and the last line of `interview`.

use std::env::temp_dir;
use std::fs;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::process;

use clap::Parser;
use clap::error::ErrorKind;
use webqa_core::config::interview::Interview;
use webqa_core::config::persona::Persona;
use webqa_core::config::purpose::{self, Purpose};
use webqa_core::interview::compare::Mismatch;
use webqa_core::interview::ledger::{Head, Ledger};
use webqa_core::ports::StoreError;
use webqa_core::redact::Redactor;

use super::*;
use crate::cli::{Cli, Command, InterviewArgs, Jobless, Sub};
use crate::interview::reset::ResetError;
use crate::run::RunError;

fn read(args: &[&str]) -> Result<Cli, ErrorKind> {
    Cli::try_parse_from(args).map_err(|e| e.kind())
}

#[test]
fn interview_arguments_are_read() {
    let read_ok = |args: &[&str]| read(args).map(|cli| cli.command);
    let args = |out: &str, previous: Option<&str>| {
        Sub::Exec(Command::Jobless(Jobless::Interview(InterviewArgs {
            config: "a".into(),
            out: out.into(),
            previous: previous.map(Into::into),
        })))
    };
    let full = ["webqa", "interview", "--config", "a", "--out", "b"];
    assert_eq!(
        read_ok(&[full.as_slice(), &["--previous", "c"]].concat()),
        Ok(args("b", Some("c")))
    );
    let bare = ["webqa", "interview", "--config", "a"];
    assert_eq!(read_ok(&bare), Ok(args(".webqa", None)));
}

#[test]
fn interview_endings_are_described() {
    let dir = Path::new("runs/x");
    let wrote = |stopped| Ending::Wrote {
        stopped,
        confirmed: false,
    };
    let line = |ending| describe(ending, dir);
    let said = [Stopped::Completed, Stopped::TimeUp, Stopped::Interrupted]
        .map(|stopped| line(wrote(stopped)));
    assert_eq!(
        said,
        [
            "終了理由 completed、台帳 runs/x/ledger.md",
            "終了理由 time_up、台帳 runs/x/ledger.md",
            "終了理由 interrupted、台帳 runs/x/ledger.md",
        ]
    );
    let reset = Ending::ResetFailed(ResetError::TimedOut);
    assert_eq!(
        line(reset),
        "初期化のコマンドが待ち時間内に終わらない、台帳 runs/x/ledger.md"
    );
    let mismatch = Ending::Mismatch(Mismatch::Goal);
    assert_eq!(line(mismatch), Mismatch::Goal.to_string());
    let failed = Ending::Failed(RunError::Store(StoreError));
    assert_eq!(line(failed), StoreError.to_string());
    assert_eq!(line(Ending::Abandoned), "記録を書き終える前に中断した");
}

fn interview(purpose: Purpose) -> Interview {
    let one = NonZeroU32::MIN;
    Interview {
        persona: Persona {
            name: "佐藤さん".to_owned(),
            profile: "60代".to_owned(),
            it_literacy: "基本操作だけ".to_owned(),
            wants: "注文する".to_owned(),
            avoids: vec!["URL を手で打たない".to_owned()],
        },
        purpose,
        trials: one,
        hesitation_threshold: 0.6,
        confidence_threshold: 0.5,
        stuck_threshold: one,
        requests_per_hour: one,
        max_natural_questions: one,
        reset: None,
    }
}

/// The path of a file holding `text`, in a directory of this test run.
fn file(name: &str, text: &str) -> PathBuf {
    let dir = temp_dir().join(format!("webqa-cli-{}", process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, text).unwrap();
    path
}

/// A ledger of `persona` and the written purpose `goal`.
fn ledger_text(persona: &str, goal: Goal) -> String {
    let head = Head {
        persona: persona.to_owned(),
        goal,
        start: "https://app.example.test/".to_owned(),
        finished: 1,
        images: false,
    };
    let ledger = Ledger {
        head,
        entries: Vec::new(),
        unchecked_purposes: Vec::new(),
    };
    ledger::write(&ledger, &Redactor::default())
}

/// What stops the start when `path` is the previous ledger.
async fn checking(path: &Path, interview: &Interview) -> Option<Stop> {
    previous(Some(path), interview, "https://app.example.test/")
        .await
        .err()
}

#[tokio::test]
async fn the_previous_ledger_is_read_and_checked_before_the_trials() {
    let start = "https://app.example.test/";
    let inferred = interview(Purpose::inferred());
    let said = |stop: Option<Stop>| stop.map(Stop::into_parts);
    assert_eq!(previous(None, &inferred, start).await.ok(), Some(None));
    assert_eq!(
        said(checking(Path::new("no/such/ledger"), &inferred).await),
        Some((2, "前回の台帳を読めない".to_owned()))
    );
    let broken = file("broken.md", "# 壊れた台帳\n");
    assert_eq!(
        said(checking(&broken, &inferred).await),
        Some((2, "台帳の 1 行目が決まった書式でない".to_owned()))
    );
}

#[tokio::test]
async fn the_head_of_the_previous_ledger_must_match_this_run() {
    let inferred = interview(Purpose::inferred());
    let said = |stop: Option<Stop>| stop.map(Stop::into_parts);
    let guessed = Goal::Inferred(vec!["注文する".to_owned()]);
    let other = file("other.md", &ledger_text("鈴木さん", guessed.clone()));
    assert_eq!(
        said(checking(&other, &inferred).await),
        Some((2, "前回の台帳のペルソナが今回と違う".to_owned()))
    );
    let same = file("same.md", &ledger_text("佐藤さん", guessed));
    assert_eq!(checking(&same, &inferred).await, None);
    let written = Goal::Written("注文を確定する".to_owned());
    let kept = file("written.md", &ledger_text("佐藤さん", written));
    assert_eq!(
        said(checking(&kept, &inferred).await),
        Some((2, "前回の台帳の目的が今回と違う".to_owned()))
    );
    let aim = "goal: 注文を確定する\ndone_when:\n  url: /done\n";
    let same_goal = interview(purpose::parse(aim).unwrap());
    assert_eq!(checking(&kept, &same_goal).await, None);
}
