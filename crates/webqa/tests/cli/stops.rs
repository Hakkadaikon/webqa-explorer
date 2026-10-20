//! The tests of `interview` that stop before or between trials.

use std::fs;
use std::io;

use super::case::{Case, arg};
use super::config::Config;
use super::exec;
use super::interview::{DOCS, interview_run, interviewed, tail};

/// `interview` of two trials with `config`, which stops with `code` and says
/// `said` before any trial ends.
fn stopped(case: &Case, config: &Config<'_>, code: i32, said: &str) -> io::Result<()> {
    let output = case.run(&["interview"], config, &[])?;
    assert_eq!(exec::code(&output), Some(code), "exit code");
    assert_eq!(
        exec::stderr(&output),
        said,
        "the sentence on the standard error"
    );
    Ok(())
}

#[test]
fn a_missing_configuration_file_exits_two() -> io::Result<()> {
    let case = Case::new("interview_no_config")?;
    let missing = arg(&case.dir.join("no-such.yaml"));
    let output = exec::run(&case.dir, &["interview", "--config", &missing])?;
    assert_eq!(exec::code(&output), Some(2), "exit code");
    assert_eq!(exec::stderr(&output), "設定ファイルを読めない\n");
    Ok(())
}

#[test]
fn a_judge_that_cannot_start_exits_three() -> io::Result<()> {
    let case = Case::new("interview_judge_spawn")?;
    let mut config = case.config("done");
    let tail = tail("persona.yaml", 2);
    config.tail = &tail;
    config.command = Some("/no/such/judge");
    stopped(&case, &config, 3, "橋渡しのコマンドを起動できない\n")
}

#[test]
fn an_output_that_cannot_be_made_exits_three() -> io::Result<()> {
    let case = Case::new("interview_no_out")?;
    fs::write(case.out(), "a file, not a directory")?;
    let mut config = case.config("done");
    let tail = tail("persona.yaml", 2);
    config.tail = &tail;
    stopped(&case, &config, 3, "記録の出力先を作れない\n")
}

/// A reset that removes the persona document, which the browser's launch
/// reads again, stops the trial that follows it.
#[test]
fn a_browser_that_cannot_be_launched_ends_the_interview_as_failed() -> io::Result<()> {
    let case = Case::new("interview_launch_fails")?;
    let persona = case.dir.join("persona.yaml");
    fs::copy(format!("{DOCS}persona.yaml"), &persona)?;
    let persona = arg(&persona);
    let mut config = case.config("done");
    let tail = format!(
        "interview:\n  persona: {persona}\n  trials: 2\n  reset:\n    command: [\"rm\", \"{persona}\"]\n"
    );
    config.tail = &tail;
    stopped(
        &case,
        &config,
        3,
        "interview.persona のファイルを読めない\nブラウザを操作できない\n",
    )
}

#[test]
fn an_interview_report_cannot_be_replayed() -> io::Result<()> {
    let case = Case::new("interview_replay")?;
    interviewed(&case, &[])?;
    let report = arg(&interview_run(&case)?.join("report.json"));
    let config = arg(&case.dir.join("config.yaml"));
    let args = ["replay", &report, "--config", &config];
    let output = exec::run(&case.dir, &args)?;
    assert_eq!(exec::code(&output), Some(2), "exit code");
    assert_eq!(
        exec::stderr(&output),
        "インタビューの実行結果は再実行できない\n"
    );
    Ok(())
}
