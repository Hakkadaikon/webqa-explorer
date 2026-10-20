//! The runs of the mode tests.

use std::fs;
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

/// What the report of a replay of `original` scripts, run into the same
/// output directory and the same site as the original. The judge of the
/// configuration is never started. The replay makes the same decisions as the
/// original and leaves the stored memory as it was.
pub fn replayed(original: &Path) -> io::Result<Script> {
    let case = case_of(original)?;
    let out = case.out();
    let out = out.as_path();
    let mut config = case.config("silent");
    config.max_decisions = 2;
    config.max_duration = 60;
    let before = memory_files(out);
    let asked = fs::read_to_string(&case.log).ok();
    let report = arg(&original.join("report.json"));
    let output = case.run(&["replay", &report], &config, &[])?;
    assert_eq!(exec::code(&output), Some(0), "exit code");
    assert_eq!(
        fs::read_to_string(&case.log).ok(),
        asked,
        "the bridge was not started"
    );
    assert_eq!(memory_files(out), before, "the stored memory is left alone");
    let again = exec::latest_run(out)?;
    assert_eq!(decided(&again)?, decided(original)?, "the same decisions");
    Ok(script(&case.report()?))
}

/// The case that made the run `original`: its directory and its site.
fn case_of(original: &Path) -> io::Result<Case> {
    let missing = || io::Error::from(io::ErrorKind::NotFound);
    let dir = original.ancestors().nth(3).ok_or_else(missing)?;
    let dir = dir.to_path_buf();
    let report = exec::report(original)?;
    let url = field(&report, "target").ok_or_else(missing)?.to_owned();
    Ok(Case::of(dir, url))
}

/// The memory files under `out`, by name, as bytes.
fn memory_files(out: &Path) -> Vec<(String, Vec<u8>)> {
    let dir = out.join("memory");
    let named = exec::names(&dir).into_iter();
    named
        .map(|name| (fs::read(dir.join(&name)).unwrap_or_default(), name))
        .map(|(bytes, name)| (name, bytes))
        .collect()
}

/// The operation and target of each decision in the events of `run`.
fn decided(run: &Path) -> io::Result<Vec<(Value, Value)>> {
    let events = exec::events(run)?;
    let decisions = events
        .iter()
        .filter(|e| field(e, "kind") == Some("decided"));
    Ok(decisions
        .map(|e| (e["op"].clone(), e["target"].clone()))
        .collect())
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
