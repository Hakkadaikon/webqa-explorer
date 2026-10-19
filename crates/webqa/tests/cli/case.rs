//! A test's working directory, and what the tests check in it.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

use serde_json::Value;

use super::config::Config;
use super::exec::{self, Running};
use super::site;

/// One test's working directory, site and bridge log.
pub struct Case {
    pub dir: PathBuf,
    pub url: String,
    pub log: PathBuf,
}

impl Case {
    pub fn new(name: &str) -> io::Result<Self> {
        let dir = exec::workdir(name)?;
        let log = dir.join("bridge.log");
        Ok(Self {
            dir,
            url: site::serve()?,
            log,
        })
    }

    pub fn out(&self) -> PathBuf {
        self.dir.join("out")
    }

    pub fn config<'a>(&'a self, mode: &'a str) -> Config<'a> {
        Config::new(&self.url, mode, &self.log)
    }

    /// The arguments of `subcommand`, with the configuration written from `config`.
    fn args(
        &self,
        subcommand: &[&str],
        config: &Config<'_>,
        extra: &[&str],
    ) -> io::Result<Vec<String>> {
        let path = config.write(&self.dir)?;
        let mut args: Vec<String> = subcommand.iter().map(ToString::to_string).collect();
        args.extend(["--config".to_owned(), path.display().to_string()]);
        args.extend(["--out".to_owned(), self.out().display().to_string()]);
        args.extend(extra.iter().map(ToString::to_string));
        Ok(args)
    }

    pub fn start(
        &self,
        subcommand: &[&str],
        config: &Config<'_>,
        extra: &[&str],
    ) -> io::Result<Running> {
        exec::start(&self.dir, &self.args(subcommand, config, extra)?)
    }

    pub fn run(
        &self,
        subcommand: &[&str],
        config: &Config<'_>,
        extra: &[&str],
    ) -> io::Result<Output> {
        self.start(subcommand, config, extra)?.finish()
    }

    pub fn explore(&self, config: &Config<'_>) -> io::Result<Output> {
        self.run(&["explore"], config, &[])
    }

    /// The latest run's report.
    pub fn report(&self) -> io::Result<Value> {
        exec::report(&exec::latest_run(&self.out())?)
    }

    pub fn runs_exist(&self) -> bool {
        self.out().join("runs").exists()
    }

    /// The first and last lines of the events of the latest run.
    pub fn ends(&self) -> io::Result<(Option<String>, Option<String>)> {
        let events = exec::events(&exec::latest_run(&self.out())?)?;
        let first = events.first().and_then(|e| field(e, "kind"));
        let last = events.last().and_then(|e| field(e, "reason"));
        Ok((first.map(str::to_owned), last.map(str::to_owned)))
    }

    /// The number of memory files.
    pub fn memories(&self) -> usize {
        exec::names(&self.out().join("memory")).len()
    }

    /// Whether the bridge was asked at least once.
    pub fn asked(&self) -> bool {
        fs::read_to_string(&self.log).is_ok_and(|text| text.contains("asked"))
    }
}

pub fn field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

/// A refused start: exit code 2, nothing written, the bridge not started.
pub fn assert_refused(case: &Case, output: &Output) {
    assert_eq!(exec::code(output), Some(2), "exit code");
    assert!(!case.runs_exist(), "no run directory");
    assert!(!case.log.exists(), "the bridge was not started");
    assert_ne!(exec::stderr(output), "", "a sentence on the standard error");
}

/// Waits until the bridge was asked, then interrupts `running`.
pub fn interrupt_when_asked(case: &Case, running: &Running) -> io::Result<()> {
    for _ in 0..1200 {
        if case.asked() {
            return interrupt(running.pid);
        }
        let early = running.wait(Duration::from_millis(50))?;
        assert!(early.is_none(), "webqa ended before the bridge was asked");
    }
    Err(io::Error::from(io::ErrorKind::TimedOut))
}

fn interrupt(pid: u32) -> io::Result<()> {
    let status = Command::new("kill")
        .args(["-INT", &pid.to_string()])
        .status()?;
    assert!(status.success(), "kill sent the signal");
    Ok(())
}

/// The `op` and `target` of each step, and the `answers` of each call.
pub fn script(report: &Value) -> (Vec<(Value, Value)>, Vec<Value>) {
    let list = |key: &str| report.get(key).and_then(Value::as_array).cloned();
    let steps = list("steps").unwrap_or_default();
    let ops = steps.iter().map(|s| (s["op"].clone(), s["target"].clone()));
    let calls = list("calls").unwrap_or_default();
    (
        ops.collect(),
        calls.iter().map(|c| c["answers"].clone()).collect(),
    )
}

/// The kinds of the issues of `report`.
pub fn issue_kinds(report: &Value) -> Vec<String> {
    let issues = report.get("issues").and_then(Value::as_array);
    let kinds = issues.map(|all| all.iter().filter_map(|i| field(i, "kind")));
    kinds
        .map(|kinds| kinds.map(str::to_owned).collect())
        .unwrap_or_default()
}

/// The path of `file`, as an argument.
pub fn arg(file: &Path) -> String {
    file.display().to_string()
}
