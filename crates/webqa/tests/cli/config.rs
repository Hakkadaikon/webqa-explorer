//! The configuration the executable tests write.

use std::fs;
use std::path::{Path, PathBuf};

/// The bridge the judge command starts.
pub const BRIDGE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cli/bridge.sh");

/// What a test changes in the configuration.
pub struct Config<'a> {
    /// The start URL.
    pub url: &'a str,
    /// How the bridge answers.
    pub mode: &'a str,
    /// The bridge's log file.
    pub log: &'a Path,
    /// Extra lines inside `judge`, indented by the caller.
    pub judge_extra: &'a str,
    /// The command of the judge, replacing the bridge.
    pub command: Option<&'a str>,
    /// `judge.kind`.
    pub kind: &'a str,
    /// `limits.freeze_threshold`.
    pub freeze: u32,
    /// `limits.max_decisions`.
    pub max_decisions: u32,
    /// An `ops:` line, or empty.
    pub ops: &'a str,
}

impl<'a> Config<'a> {
    /// The usual configuration of the tests.
    pub fn new(url: &'a str, mode: &'a str, log: &'a Path) -> Self {
        Config {
            url,
            mode,
            log,
            judge_extra: "",
            command: None,
            kind: "command",
            freeze: 1000,
            max_decisions: 10000,
            ops: "",
        }
    }

    /// Writes the file in `dir` and returns its path.
    pub fn write(&self, dir: &Path) -> PathBuf {
        let command = self.command.map_or_else(
            || format!(r#"["{BRIDGE}", "{}", "{}"]"#, self.mode, self.log.display()),
            |c| format!(r#"["{c}"]"#),
        );
        let yaml = format!(
            r#"target:
  url: {url}
  allowed_origins: []
judge:
  kind: {kind}
  command: {command}
  timeout_secs: 30
  pricing: {{ input_per_1k: 0.0, output_per_1k: 0.0 }}
{extra}limits:
  freeze_threshold: {freeze}
  stuck_threshold: 1000
  max_decisions: {max_decisions}
  requests_per_hour: 10000
  page_timeout_secs: 5
  max_duration_secs: 2
  back_confidence: 0.7
  error_confidence: 0.8
guard:
  forbidden: {{ keywords: [], selectors: [] }}
  allowed: {{ selectors: [] }}
{ops}scenario:
  stages:
    - name: 開く
      goal: 開く
      done_when:
        url: /
"#,
            url = self.url,
            kind = self.kind,
            extra = self.judge_extra,
            freeze = self.freeze,
            max_decisions = self.max_decisions,
            ops = self.ops,
        );
        let path = dir.join("config.yaml");
        fs::write(&path, yaml).expect("write the configuration");
        path
    }
}
