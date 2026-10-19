//! Tests of the ledger file: selecting writes back to the same file.

mod sample;

use std::{fs, io, io::ErrorKind, path::PathBuf};

use sample::{number, sample};
use webqa_core::interview::ledger::{self, LedgerError, write};
use webqa_core::redact::Redactor;
use webqa_tui::ledger_file::{FileError, LedgerFile};

/// A new file in a directory of its own, holding `text`.
fn file_with(name: &str, text: &str) -> Result<PathBuf, io::Error> {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::create_dir_all(&dir)?;
    let path = dir.join("ledger.md");
    fs::write(&path, text)?;
    Ok(path)
}

fn sample_text() -> Option<String> {
    sample().map(|ledger| write(&ledger, &Redactor::default()))
}

#[test]
fn toggle_writes_back_to_same_file() {
    let text = sample_text().unwrap();
    let path = file_with("toggle_writes_back", &text).unwrap();
    let (mut file, _) = LedgerFile::open(&path).unwrap();
    let ledger = file.set_selection(number(2).unwrap(), true).unwrap();
    let expected = ledger::set_selection(&text, number(2).unwrap(), true).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), expected);
    assert_eq!(ledger.selected(), vec![number(2).unwrap()]);
}

#[test]
fn toggle_twice_restores_bytes() {
    let text = sample_text().unwrap();
    let path = file_with("toggle_twice", &text).unwrap();
    let (mut file, _) = LedgerFile::open(&path).unwrap();
    file.set_selection(number(2).unwrap(), true).unwrap();
    let ledger = file.set_selection(number(2).unwrap(), false).unwrap();
    assert_eq!(fs::read(&path).unwrap(), text.as_bytes());
    assert_eq!(ledger.selected(), Vec::new());
}

#[test]
fn external_edit_blocks_overwrite() {
    let text = sample_text().unwrap();
    let path = file_with("external_edit", &text).unwrap();
    let (mut file, _) = LedgerFile::open(&path).unwrap();
    let outside = ledger::set_selection(&text, number(3).unwrap(), true).unwrap();
    fs::write(&path, &outside).unwrap();
    assert_eq!(
        file.set_selection(number(2).unwrap(), true),
        Err(FileError::ChangedOutside)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), outside);
}

#[test]
fn open_rejects_non_ledger() {
    let path = file_with("open_rejects", "秘密の本文\n").unwrap();
    let error = LedgerFile::open(&path).unwrap_err();
    assert_eq!(error, FileError::Ledger(LedgerError::Malformed { line: 1 }));
    assert!(!error.to_string().contains("秘密の本文"));
    let missing = path.with_file_name("missing.md");
    let error = LedgerFile::open(&missing).unwrap_err();
    assert_eq!(error, FileError::Io(ErrorKind::NotFound));
    let text = format!("{error} {error:?}");
    assert!(!text.contains("missing") && !text.contains("open_rejects"));
}

#[test]
fn unknown_number_and_vanished_file_are_errors() {
    let path = file_with("unknown_number", &sample_text().unwrap()).unwrap();
    let (mut file, _) = LedgerFile::open(&path).unwrap();
    assert_eq!(
        file.set_selection(number(9).unwrap(), true),
        Err(FileError::Ledger(LedgerError::NoIssue(number(9).unwrap())))
    );
    fs::remove_file(&path).unwrap();
    assert_eq!(
        file.set_selection(number(1).unwrap(), true),
        Err(FileError::Io(ErrorKind::NotFound))
    );
}

#[test]
fn write_failure_is_an_io_error() {
    use std::os::unix::fs::PermissionsExt;
    let path = file_with("write_failure", &sample_text().unwrap()).unwrap();
    let (mut file, _) = LedgerFile::open(&path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();
    let result = file.set_selection(number(1).unwrap(), true);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(result, Err(FileError::Io(ErrorKind::PermissionDenied)));
}
