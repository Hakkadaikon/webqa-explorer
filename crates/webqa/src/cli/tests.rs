//! Tests of the argument reading.

use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::clock::Wall;
use crate::run::RunError;
use crate::run::session::Ending;
use jiff::Timestamp;
use webqa_core::ports::StoreError;

use clap::Parser;
use clap::error::ErrorKind;
use webqa_core::event::Mode;
use webqa_judge::table::StartError;

use super::*;

fn read(args: &[&str]) -> Result<Cli, ErrorKind> {
    Cli::try_parse_from(args).map_err(|e| e.kind())
}

fn run_args(config: &str, out: &str, max: Option<u64>) -> RunArgs {
    RunArgs {
        config: config.into(),
        out: out.into(),
        max_duration_secs: max.and_then(NonZeroU64::new),
    }
}

#[test]
fn arguments_are_read() {
    let read_ok = |args: &[&str]| read(args).map(|cli| cli.command);
    assert_eq!(
        read_ok(&["webqa", "explore", "--config", "c.yaml"]),
        Ok(Command::Explore(run_args("c.yaml", ".webqa", None)))
    );
    assert_eq!(
        read_ok(&[
            "webqa",
            "run",
            "--config",
            "c.yaml",
            "--out",
            "o",
            "--max-duration-secs",
            "60"
        ]),
        Ok(Command::Run(run_args("c.yaml", "o", Some(60))))
    );
    assert_eq!(
        read_ok(&["webqa", "replay", "r.json", "--config", "c.yaml"]),
        Ok(Command::Replay(ReplayArgs {
            report: "r.json".into(),
            config: "c.yaml".into(),
            out: ".webqa".into(),
        }))
    );
}

#[test]
fn bad_arguments_are_errors() {
    let zero = ["webqa", "run", "--config", "c", "--max-duration-secs", "0"];
    assert_eq!(read(&zero), Err(ErrorKind::ValueValidation));
    assert_eq!(read(&["webqa", "watch"]), Err(ErrorKind::InvalidSubcommand));
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
fn subcommands_become_jobs() {
    let job = |args: &[&str]| Cli::try_parse_from(args).map(|cli| start::job(cli.command));
    let expected = |source, out: &str, max| start::Job {
        source,
        config: "c.yaml".into(),
        out: out.into(),
        max_duration: max,
    };
    let sixty = NonZeroU64::new(60);
    assert_eq!(
        job(&[
            "webqa",
            "run",
            "--config",
            "c.yaml",
            "--max-duration-secs",
            "60"
        ])
        .ok(),
        Some(expected(start::Source::Scenario, ".webqa", sixty))
    );
    assert_eq!(
        job(&["webqa", "explore", "--config", "c.yaml", "--out", "o"]).ok(),
        Some(expected(start::Source::Explore, "o", None))
    );
    assert_eq!(
        job(&["webqa", "replay", "r.json", "--config", "c.yaml"]).ok(),
        Some(expected(
            start::Source::Replay("r.json".into()),
            ".webqa",
            None
        ))
    );
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
    let stop = start::setup("target: {}", Mode::Explore, None).err();
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
fn time_limit_of_the_arguments_wins() {
    let limit = |max| {
        start::setup(CONFIG, Mode::Explore, max)
            .ok()
            .map(|setup| setup.rules.max_duration)
    };
    assert_eq!(limit(None), Some(None));
    let seven = NonZeroU64::new(7);
    assert_eq!(limit(seven), Some(Some(Duration::from_secs(7))));
}

#[test]
fn a_file_that_is_no_storage_state_is_a_configuration_error() {
    let yaml = format!("{CONFIG}auth: {{ storage_state: {THIS_FILE} }}\n");
    let stop = start::setup(&yaml, Mode::Explore, None).err();
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

#[test]
fn endings_are_described() {
    let dir = Path::new("runs/x");
    let failed = Ending::Failed(RunError::Store(StoreError));
    assert_eq!(app::describe(&failed, dir), StoreError.to_string());
    assert_eq!(
        app::describe(&Ending::Abandoned, dir),
        "記録を書き終える前に中断した"
    );
}

#[test]
fn an_abandoned_runtime_does_not_wait_for_blocked_work() {
    let (release, blocked) = channel::<()>();
    let runtime = Builder::new_current_thread().build().unwrap();
    drop(runtime.spawn_blocking(move || blocked.recv()));
    stop_runtime(runtime, true);
    drop(release);
    let runtime = Builder::new_current_thread().build().unwrap();
    stop_runtime(runtime, false);
}
