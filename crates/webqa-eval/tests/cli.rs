//! `webqa-eval` as a process.

use std::io;
use std::process::Command;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

/// Runs `webqa-eval`; its exit code, output and error output.
fn webqa_eval(args: &[&str]) -> io::Result<(Option<i32>, String, String)> {
    let output = Command::new(env!("CARGO_BIN_EXE_webqa-eval"))
        .args(args)
        .output()?;
    Ok((
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

#[test]
fn score_prints_the_total_row_of_a_ledger() -> io::Result<()> {
    let truth = format!("{FIXTURES}/truth.yaml");
    let labels = format!("{FIXTURES}/labels.yaml");
    let ledger = format!("{FIXTURES}/ledger.txt");
    let (code, out, err) = webqa_eval(&["score", "--truth", &truth, "--labels", &labels, &ledger])?;
    assert_eq!(
        (code, out.lines().find(|l| l.starts_with("| 全体")), err),
        (
            Some(0),
            Some("| 全体 | 0.67 | 0.67 | 1.00 |"),
            String::new()
        )
    );
    Ok(())
}

#[test]
fn score_with_a_missing_answer_key_exits_2_with_the_reason() -> io::Result<()> {
    let labels = format!("{FIXTURES}/labels.yaml");
    let ledger = format!("{FIXTURES}/ledger.txt");
    let (code, out, err) = webqa_eval(&[
        "score",
        "--truth",
        "none.yaml",
        "--labels",
        &labels,
        &ledger,
    ])?;
    assert_eq!(
        (code, out, err),
        (
            Some(2),
            String::new(),
            "none.yaml: No such file or directory (os error 2)\n".to_owned()
        )
    );
    Ok(())
}
