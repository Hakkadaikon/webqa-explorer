//! Tests of `webqa ledger`, the executable. A real terminal is not reached:
//! these runs have no terminal on the standard input.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const LEDGER: &str = include_str!("../fixtures/ledger.txt");

fn ledger(path: &str) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_webqa"))
        .args(["ledger", path])
        .stdin(Stdio::null())
        .output()
}

fn said(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn a_ledger_without_a_terminal_exits_three() -> io::Result<()> {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ledger-no-terminal");
    fs::create_dir_all(&dir)?;
    let path = dir.join("ledger.md");
    fs::write(&path, LEDGER)?;
    let output = ledger(&path.to_string_lossy())?;
    assert_eq!(output.status.code(), Some(3));
    assert!(said(&output).contains("端末が無いので台帳の画面を開けない"));
    assert_eq!(fs::read_to_string(&path)?, LEDGER, "nothing is written");
    Ok(())
}

#[test]
fn a_ledger_that_cannot_be_opened_exits_two() -> io::Result<()> {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ledger-missing");
    let output = ledger(&dir.join("none.md").to_string_lossy())?;
    assert_eq!(output.status.code(), Some(2));
    assert!(said(&output).contains("台帳"), "{}", said(&output));
    Ok(())
}
