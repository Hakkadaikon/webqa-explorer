//! The tests of `interview` and `check-docs`.

use std::fs;
use std::io;
use std::path::Path;

use webqa_core::interview::ledger::{Goal, Head, Ledger, read, write};
use webqa_core::redact::Redactor;

use super::case::{Case, arg, assert_refused};
use super::exec;

const DOCS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/docs/");

/// The configuration of an interview of `trials` trials, with the persona
/// document `persona`.
pub fn tail(persona: &str, trials: u32) -> String {
    format!("interview:\n  persona: {DOCS}{persona}\n  trials: {trials}\n")
}

/// The ledger of a run of `persona` that wrote its purpose as `goal`, with
/// no issue.
pub fn ledger_of(persona: &str, goal: &str) -> String {
    let head = Head {
        persona: persona.to_owned(),
        goal: Goal::Written(goal.to_owned()),
        start: "https://app.example.test/".to_owned(),
        finished: 1,
        images: false,
    };
    let ledger = Ledger {
        head,
        entries: Vec::new(),
        unchecked_purposes: Vec::new(),
    };
    write(&ledger, &Redactor::default())
}

/// The standard error and exit code of `check-docs` on `file`.
fn checked(case: &Case, kind: &str, file: &str) -> io::Result<(Option<i32>, String)> {
    let path = format!("{DOCS}{file}");
    let output = exec::run(&case.dir, &["check-docs", kind, &path])?;
    Ok((exec::code(&output), exec::stderr(&output)))
}

/// `interview` with the config of `tail` and `previous`, refused before the
/// browser opens.
fn refused(case: &Case, tail: &str, previous: Option<&Path>, said: &str) -> io::Result<()> {
    let mut config = case.config("done");
    config.tail = tail;
    let previous = previous.map(arg);
    let extra: Vec<&str> = previous
        .iter()
        .flat_map(|path| ["--previous", path.as_str()])
        .collect();
    let output = case.run(&["interview"], &config, &extra)?;
    assert_refused(case, &output);
    assert_eq!(
        exec::stderr(&output),
        said,
        "the sentence on the standard error"
    );
    assert!(!case.visited(), "the site was not visited");
    Ok(())
}

#[test]
fn interview_config_error_exits_two_before_opening() -> io::Result<()> {
    let case = Case::new("interview_config_error")?;
    refused(
        &case,
        &tail("persona-no-literacy.yaml", 2),
        None,
        "persona.it_literacy を書く\n",
    )
}

#[test]
fn broken_previous_ledger_exits_two() -> io::Result<()> {
    let case = Case::new("broken_previous_ledger")?;
    let previous = case.dir.join("previous.md");
    let text = ledger_of("佐藤さん", "注文する");
    fs::write(
        &previous,
        text.replace("## 確かめられなかった課題", "## 課題の区切り"),
    )?;
    refused(
        &case,
        &tail("persona.yaml", 2),
        Some(&previous),
        "台帳の 14 行目が決まった書式でない\n",
    )
}

#[test]
fn mismatched_previous_ledger_exits_two() -> io::Result<()> {
    let case = Case::new("mismatched_previous_ledger")?;
    let previous = case.dir.join("previous.md");
    fs::write(&previous, ledger_of("鈴木さん", "注文する"))?;
    let said = "前回の台帳のペルソナが今回と違う\n";
    refused(&case, &tail("persona.yaml", 2), Some(&previous), said)
}

#[test]
fn check_accepts_valid_documents() -> io::Result<()> {
    let case = Case::new("check_accepts")?;
    assert_eq!(
        checked(&case, "persona", "persona.yaml")?,
        (Some(0), String::new())
    );
    assert_eq!(
        checked(&case, "purpose", "purpose.yaml")?,
        (Some(0), String::new())
    );
    Ok(())
}

#[test]
fn check_reports_field_and_exits_two() -> io::Result<()> {
    let case = Case::new("check_reports")?;
    let persona = checked(&case, "persona", "persona-no-literacy.yaml")?;
    assert_eq!(
        persona,
        (Some(2), "persona.it_literacy を書く\n".to_owned())
    );
    let purpose = checked(&case, "purpose", "purpose-no-declaration.yaml")?;
    assert_eq!(
        purpose,
        (
            Some(2),
            "purpose.done_when に url、element、text のどれかを1つ以上書く\n".to_owned()
        )
    );
    Ok(())
}

/// Runs `interview` with two trials, and says it ended with exit code 0.
fn interviewed(case: &Case, extra: &[&str]) -> io::Result<()> {
    let mut config = case.config("done");
    let tail = tail("persona.yaml", 2);
    config.tail = &tail;
    config.max_duration = 30;
    let output = case.run(&["interview"], &config, extra)?;
    assert_eq!(exec::code(&output), Some(0), "{}", exec::stderr(&output));
    Ok(())
}

#[test]
fn interview_runs_and_writes_ledger() -> io::Result<()> {
    let case = Case::new("interview_runs")?;
    interviewed(&case, &[])?;
    let path = exec::latest_run(&case.out())?.join("ledger.md");
    let text = fs::read_to_string(&path)?;
    assert_eq!(read(&text).map(|ledger| ledger.head.finished), Ok(2));
    // The ledger it wrote can be read back as the previous one.
    interviewed(&case, &["--previous", &arg(&path)])
}
