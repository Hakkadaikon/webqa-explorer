//! Tests of the argument reading.

use clap::Parser;
use clap::error::ErrorKind;

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
