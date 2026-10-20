//! Tests of the argument reading.

use std::path::Path;

use crate::clock::Wall;
use jiff::Timestamp;

use clap::Parser;
use clap::error::ErrorKind;
use webqa_core::event::Mode;
use webqa_judge::table::StartError;

use super::*;

fn read(args: &[&str]) -> Result<Cli, ErrorKind> {
    Cli::try_parse_from(args).map_err(|e| e.kind())
}

fn run_args(config: &str, out: &str) -> RunArgs {
    RunArgs {
        config: config.into(),
        out: out.into(),
    }
}

#[test]
fn arguments_are_read() {
    let read_ok = |args: &[&str]| read(args).map(|cli| cli.command);
    assert_eq!(
        read_ok(&["webqa", "explore", "--config", "c.yaml"]),
        Ok(Sub::Exec(Command::Explore(run_args("c.yaml", ".webqa"))))
    );
    assert_eq!(
        read_ok(&["webqa", "run", "--config", "c.yaml", "--out", "o"]),
        Ok(Sub::Exec(Command::Run(run_args("c.yaml", "o"))))
    );
    assert_eq!(
        read_ok(&["webqa", "replay", "r.json", "--config", "c.yaml"]),
        Ok(Sub::Exec(Command::Replay(ReplayArgs {
            report: "r.json".into(),
            config: "c.yaml".into(),
            out: ".webqa".into(),
        })))
    );
    assert_eq!(
        read_ok(&["webqa", "watch", ".webqa/runs/r1"]),
        Ok(Sub::Watch(WatchArgs {
            dir: ".webqa/runs/r1".into()
        }))
    );
    assert_eq!(
        read_ok(&["webqa", "ledger", "out/ledger.md"]),
        Ok(Sub::Ledger(LedgerArgs {
            path: "out/ledger.md".into()
        }))
    );
}

#[test]
fn bad_arguments_are_errors() {
    let limit = ["webqa", "run", "--config", "c", "--max-duration-secs", "60"];
    assert_eq!(read(&limit), Err(ErrorKind::UnknownArgument));
    assert_eq!(
        read(&["webqa", "watch"]),
        Err(ErrorKind::MissingRequiredArgument)
    );
    assert_eq!(
        read(&["webqa", "ledger"]),
        Err(ErrorKind::MissingRequiredArgument)
    );
    assert_eq!(read(&["webqa", "tail"]), Err(ErrorKind::InvalidSubcommand));
    assert_eq!(
        read(&["webqa", "run"]),
        Err(ErrorKind::MissingRequiredArgument)
    );
}

#[test]
fn start_errors_map_to_codes() {
    let table = [
        (StartError::UnknownKind("x".into()), 2),
        (StartError::NoCommand, 2),
        (StartError::Spawn, 3),
        (StartError::Declaration, 3),
        (StartError::Version, 3),
    ];
    for (error, code) in table {
        assert_eq!(start::judge_stop(error).into_parts().0, code);
    }
}

#[test]
fn an_unknown_judge_kind_is_said_without_its_name() {
    let stop = start::judge_stop(StartError::UnknownKind("sk-secret".into()));
    assert_eq!(
        stop.into_parts(),
        (2, "判断モデルの種類が表に無い".to_owned())
    );
}

#[test]
fn subcommands_become_jobs() {
    let job = |args: &[&str]| {
        let cli = Cli::try_parse_from(args).ok();
        cli.map(|cli| cli.command.split().map(|c| start::job(c).ok()))
    };
    let expected = |source, out: &str| start::Job {
        source,
        config: "c.yaml".into(),
        out: out.into(),
    };
    assert_eq!(
        job(&["webqa", "run", "--config", "c.yaml"]),
        Some(Ok(Some(expected(start::Source::Scenario, ".webqa"))))
    );
    assert_eq!(
        job(&["webqa", "explore", "--config", "c.yaml", "--out", "o"]),
        Some(Ok(Some(expected(start::Source::Explore, "o"))))
    );
    assert_eq!(
        job(&["webqa", "replay", "r.json", "--config", "c.yaml"]),
        Some(Ok(Some(expected(
            start::Source::Replay("r.json".into()),
            ".webqa"
        ))))
    );
    assert_eq!(
        job(&["webqa", "watch", "d"]),
        Some(Err(Viewing::Watch(WatchArgs { dir: "d".into() }))),
        "watching starts no run"
    );
}

#[test]
fn the_ledger_screen_starts_no_run() {
    let cli = Cli::try_parse_from(["webqa", "ledger", "l.md"]).ok();
    let split = cli.map(|cli| cli.command.split().map(|c| start::job(c).ok()));
    let screen = Viewing::Ledger(LedgerArgs {
        path: "l.md".into(),
    });
    assert_eq!(split, Some(Err(screen)));
}

#[test]
fn other_subcommands_become_no_jobs() {
    let job = |args: &[&str]| {
        let cli = Cli::try_parse_from(args).ok();
        cli.map(|cli| cli.command.split().map(|c| start::job(c).ok()))
    };
    let docs = ["webqa", "check-docs", "persona", "a"];
    assert_eq!(job(&docs), Some(Ok(None)));
    let interview = ["webqa", "interview", "--config", "a"];
    assert_eq!(job(&interview), Some(Ok(None)));
}

#[tokio::test]
async fn unreadable_files_stop_as_configuration_errors() {
    let missing = Path::new("no/such/file");
    assert_eq!(
        start::config_text(missing).await,
        Err(start::Stop::Config("設定ファイルを読めない".into()))
    );
    let replay = start::script(&start::Source::Replay(missing.into())).await;
    assert_eq!(
        replay.err(),
        Some(start::Stop::Config("実行結果を読めない".into()))
    );
}

#[tokio::test]
async fn modes_come_from_the_subcommand() {
    let mode = |source| async move { start::script(&source).await.map(|(mode, _)| mode) };
    assert_eq!(
        mode(start::Source::Scenario).await.ok(),
        Some(Mode::Scenario)
    );
    assert_eq!(mode(start::Source::Explore).await.ok(), Some(Mode::Explore));
}

#[test]
fn a_bad_configuration_stops_with_code_two() {
    let stop = start::setup("target: {}", Mode::Explore).err();
    assert_eq!(stop.map(|s| s.into_parts().0), Some(2));
}

const CONFIG: &str = "
target: { url: 'http://localhost:1/', allowed_origins: [] }
judge:
  kind: command
  command: ['x']
  timeout_secs: 1
  pricing: { input_per_1k: 0.0, output_per_1k: 0.0 }
limits:
  freeze_threshold: 1
  stuck_threshold: 1
  max_decisions: 1
  requests_per_hour: 1
  page_timeout_secs: 1
  back_confidence: 0.7
  error_confidence: 0.8
guard:
  forbidden: { keywords: [], selectors: [] }
  allowed: { selectors: [] }
";

const THIS_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");

#[test]
fn a_file_that_is_no_storage_state_is_a_configuration_error() {
    let yaml = format!("{CONFIG}auth: {{ storage_state: {THIS_FILE} }}\n");
    let stop = start::setup(&yaml, Mode::Explore).err();
    assert_eq!(stop.map(|s| s.into_parts().0), Some(2));
}

#[tokio::test]
async fn a_file_that_is_no_report_stops_as_a_configuration_error() {
    let replay = start::script(&start::Source::Replay(THIS_FILE.into())).await;
    assert_eq!(
        replay.err().map(start::Stop::into_parts),
        Some((2, "実行結果を再実行の台本として読めない".to_owned()))
    );
}

#[tokio::test]
async fn an_output_that_cannot_be_made_stops_as_a_runtime_error() {
    let wall = Wall::at(Timestamp::UNIX_EPOCH);
    let out = Path::new(THIS_FILE).join("out");
    assert_eq!(
        start::store(&out, &wall).await.err(),
        Some(start::Stop::Runtime("記録の出力先を作れない".into()))
    );
}

mod ending;
