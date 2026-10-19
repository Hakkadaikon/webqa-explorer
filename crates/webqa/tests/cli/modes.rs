//! The runs of the mode tests.

use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

use super::case::{Case, arg, assert_refused, field, issue_kinds, script};
use super::exec;

type Script = (Vec<(Value, Value)>, Vec<Value>);

/// `run` with a judge that would say stuck: the stage is done at the start.
pub fn scenario(case: &Case) -> io::Result<()> {
    let output = case.run(&["run"], &case.config("stuck"), &[])?;
    assert_eq!(exec::code(&output), Some(0), "exit code");
    let report = case.report()?;
    let seen = (field(&report, "reason"), field(&report, "mode"));
    assert_eq!(
        seen,
        (Some("goal_achieved"), Some("scenario")),
        "the scenario run reached its goal"
    );
    let expected = (Some("run_started".into()), Some("goal_achieved".into()));
    assert_eq!(case.ends()?, expected, "the first and last events");
    Ok(())
}

/// `explore` with a judge that waits: it runs out of time, well within 30 seconds.
pub fn explore(case: &Case) -> io::Result<()> {
    let running = case.start(&["explore"], &case.config("wait"), &[])?;
    let ended = running.wait(Duration::from_secs(30))?;
    assert_eq!(ended.as_ref().and_then(exec::code), Some(0), "exit code");
    let report = case.report()?;
    let seen = (field(&report, "reason"), field(&report, "mode"));
    assert_eq!(
        seen,
        (Some("time_exceeded"), Some("explore")),
        "the exploration ran out of time"
    );
    assert!(case.asked(), "the judge was asked during the run");
    // The scenario run before it has the same target, so one memory file.
    assert_eq!(case.memories(), 1, "one memory file for one target");
    Ok(())
}

/// `explore` that clicks; the run directory and what its report scripts.
pub fn explore_clicking(case: &Case) -> io::Result<(PathBuf, Script)> {
    let mut config = case.config("click");
    config.max_decisions = 2;
    config.max_duration = 60;
    let output = case.explore(&config)?;
    assert_eq!(exec::code(&output), Some(0), "exit code");
    let original = exec::latest_run(&case.out())?;
    let first = script(&exec::report(&original)?);
    Ok((original, first))
}

/// What the report of a replay of `original` scripts. The judge of the
/// configuration is never started.
pub fn replayed(original: &Path) -> io::Result<Script> {
    // A fresh output directory: the memory of the first run would steer the
    // replay away from `保存`, which that run already tried. Replaying with
    // the same directory is a separate issue.
    let other = Case::new("replay_again")?;
    let mut config = other.config("silent");
    config.max_decisions = 2;
    config.max_duration = 60;
    let report = arg(&original.join("report.json"));
    let output = other.run(&["replay", &report], &config, &[])?;
    assert_eq!(exec::code(&output), Some(0), "exit code");
    assert!(!other.log.exists(), "the bridge was not started");
    Ok(script(&other.report()?))
}

/// A configuration with `extra` in `judge` is refused, and the key it may
/// hold is not echoed.
pub fn refuse_extra(extra: &str) -> io::Result<()> {
    let case = Case::new("config_error_extra")?;
    let mut config = case.config("stuck");
    config.judge_extra = extra;
    let output = case.explore(&config)?;
    assert_refused(&case, &output);
    assert!(!exec::stderr(&output).contains("sk-live-123"), "no key");
    Ok(())
}

/// A freeze threshold of 0 is refused.
pub fn refuse_freeze_zero() -> io::Result<()> {
    let case = Case::new("config_error_freeze")?;
    let mut config = case.config("stuck");
    config.freeze = 0;
    assert_refused(&case, &case.explore(&config)?);
    Ok(())
}

/// The exit code of `--help` and whether it names each subcommand.
pub fn help_names(case: &Case) -> io::Result<(Option<i32>, [bool; 3])> {
    let help = exec::run(&case.dir, &["--help"])?;
    let text = String::from_utf8_lossy(&help.stdout).into_owned();
    let named = ["run", "explore", "replay"].map(|name| text.contains(name));
    Ok((exec::code(&help), named))
}

/// The exit code and issue kinds of an exploration whose judge answers as
/// `mode` and whose configuration trusts the error display from `confidence`.
pub fn semantic(mode: &str, confidence: f64) -> io::Result<(Option<i32>, Vec<String>)> {
    let case = Case::new("semantic_findings")?;
    let mut config = case.config(mode);
    config.max_decisions = 3;
    config.max_duration = 60;
    config.error_confidence = confidence;
    let output = case.explore(&config)?;
    Ok((exec::code(&output), issue_kinds(&case.report()?)))
}
