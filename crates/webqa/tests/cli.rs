//! Tests of the `webqa` executable, with a real Chromium.

#[path = "cli/case.rs"]
mod case;
#[path = "cli/config.rs"]
mod config;
#[path = "cli/exec.rs"]
mod exec;
#[path = "cli/interview.rs"]
mod interview;
#[path = "cli/ledger.rs"]
mod ledger;
#[path = "cli/modes.rs"]
mod modes;
#[path = "cli/site.rs"]
mod site;
#[path = "cli/stops.rs"]
mod stops;

use std::env::var;
use std::fs;
use std::io;

use case::{Case, assert_refused, field};
use config::Config;
use serde_json::Value;

fn reason(report: &Value) -> Option<&str> {
    field(report, "reason")
}

#[test]
fn config_error_exits_two() -> io::Result<()> {
    assert_eq!(var("WEBQA_CLI_TEST_MISSING_KEY").ok(), None);
    let extras = [
        "  api_key: sk-live-123\n",
        "  api_key: { env: WEBQA_CLI_TEST_MISSING_KEY }\n",
        "  tls_verify: false\n",
    ];
    extras.into_iter().try_for_each(modes::refuse_extra)?;
    modes::refuse_freeze_zero()
}

#[test]
fn bad_command_lines_exit_two() -> io::Result<()> {
    let case = Case::new("bad_command_lines")?;
    let code = |args: &[&str]| exec::run(&case.dir, args).map(|output| exec::code(&output));
    assert_eq!(code(&["explore", "--config", "nothing.yaml"])?, Some(2));
    assert_eq!(code(&["wander"])?, Some(2));
    assert_eq!(modes::help_names(&case)?, (Some(0), [true; 3]));
    Ok(())
}

#[test]
fn broken_env_file_exits_two() -> io::Result<()> {
    let case = Case::new("broken_env_file")?;
    fs::write(case.dir.join(".env"), "A=\"unclosed\n")?;
    let output = exec::run(&case.dir, &["--help"])?;
    assert_eq!(exec::code(&output), Some(2));
    assert_ne!(exec::stderr(&output), "");
    Ok(())
}

#[test]
fn explore_without_click_or_input_exits_two() -> io::Result<()> {
    let case = Case::new("explore_without_click")?;
    let mut config = case.config("done");
    config.ops = "ops: [done]\n";
    assert_refused(&case, &case.explore(&config)?);
    let output = case.run(&["run"], &config, &[])?;
    assert_eq!(exec::code(&output), Some(0));
    Ok(())
}

#[test]
fn runs_scenario_and_explore_modes() -> io::Result<()> {
    let case = Case::new("runs_scenario_and_explore_modes")?;
    modes::scenario(&case)?;
    modes::explore(&case)
}

#[test]
fn interrupt_writes_records_then_exits() -> io::Result<()> {
    let case = Case::new("interrupt")?;
    let mut config = case.config("silent");
    config.max_duration = 60;
    let running = case.start(&["explore"], &config, &[])?;
    case::interrupt_when_asked(&case, &running)?;
    let output = running.finish()?;
    assert_eq!(exec::code(&output), Some(0));
    assert_eq!(reason(&case.report()?), Some("user_aborted"));
    assert_eq!(case.ends()?.1, Some("user_aborted".into()));
    assert_eq!(case.memories(), 1);
    Ok(())
}

#[test]
fn replay_reruns_report() -> io::Result<()> {
    let case = Case::new("replay")?;
    let (original, first) = modes::explore_clicking(&case)?;
    // The second question finds `保存` already tried, so it makes no step.
    assert_eq!((first.0.len(), first.1.len()), (1, 2));
    assert_eq!(modes::replayed(&original)?, first);
    Ok(())
}

#[test]
fn judge_start_failure_exits() -> io::Result<()> {
    let case = Case::new("judge_start_failure")?;
    let mut config = case.config("stuck");
    config.command = Some("/nonexistent/bridge");
    let output = case.explore(&config)?;
    assert_eq!(exec::code(&output), Some(3));
    assert!(!case.runs_exist(), "no run directory");
    let mut config = case.config("stuck");
    config.kind = "other";
    assert_refused(&case, &case.explore(&config)?);
    Ok(())
}

#[test]
fn semantic_findings_use_the_configured_confidence() -> io::Result<()> {
    let shown = modes::semantic("broken", 0.8)?;
    assert_eq!(shown, (Some(1), vec!["error_display".to_owned()]));
    // 0.9 is below the 0.95 that the configuration asks for.
    let weak = modes::semantic("broken", 0.95)?;
    assert_eq!(weak, (Some(0), Vec::<String>::new()));
    let confusing = modes::semantic("confusing", 0.8)?;
    assert_eq!(
        confusing.1.first().map(String::as_str),
        Some("semantic/わかりにくい表示")
    );
    Ok(())
}

#[test]
fn a_replay_that_cannot_be_read_exits_two() -> io::Result<()> {
    let case = Case::new("replay_unreadable")?;
    let no_report = case.dir.join("no-report.json");
    fs::write(&no_report, "{}")?;
    let report = no_report.display().to_string();
    let output = case.run(&["replay", &report], &case.config("stuck"), &[])?;
    assert_eq!(exec::code(&output), Some(2));
    assert!(!case.runs_exist(), "no run directory");
    let missing = case.dir.join("missing.json").display().to_string();
    let output = case.run(&["replay", &missing], &case.config("stuck"), &[])?;
    assert_eq!(exec::code(&output), Some(2));
    Ok(())
}

#[test]
fn a_browser_that_cannot_start_exits_three() -> io::Result<()> {
    let case = Case::new("browser_start_failure")?;
    // Nothing listens on port 1.
    let config = Config::new("http://127.0.0.1:1/", "stuck", &case.log);
    let output = case.explore(&config)?;
    assert_eq!(exec::code(&output), Some(3));
    assert!(!case.runs_exist(), "no run directory");
    Ok(())
}

#[test]
fn an_output_that_cannot_be_made_exits_three() -> io::Result<()> {
    let case = Case::new("store_start_failure")?;
    let file = case.dir.join("file");
    fs::write(&file, "")?;
    let config = case.config("stuck");
    let path = config.write(&case.dir)?;
    let args = [
        "explore",
        "--config",
        &case::arg(&path),
        "--out",
        &case::arg(&file.join("out")),
    ];
    let output = exec::run(&case.dir, &args)?;
    assert_eq!(exec::code(&output), Some(3));
    Ok(())
}
