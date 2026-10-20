//! Tests of the ledger screen's session, with scripted keys.

use std::env::temp_dir;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::id;

use webqa_core::interview::issue::{Kind, Number};
use webqa_core::interview::ledger::{self, Ledger};
use webqa_tui::keys::Key;
use webqa_tui::ledger::Notice;
use webqa_tui::ledger_file::FileError;

use super::*;

mod screen;

const FIXTURE: &str = include_str!("../../tests/fixtures/ledger.txt");

/// A new ledger file holding `text`, in a directory of its own.
fn file_with(name: &str, text: &str) -> PathBuf {
    let dir = temp_dir().join(format!("webqa-ledger-{name}-{}", id()));
    drop(fs::remove_dir_all(&dir));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("ledger.md");
    fs::write(&path, text).unwrap();
    path
}

/// The terminal size, not known.
fn unknown() -> Option<(u16, u16)> {
    None
}

fn number(n: u8) -> Number {
    Number::parse(&format!("I-0000000{n}")).unwrap()
}

fn script(keys: Vec<Key>) -> impl Keys {
    let mut keys = keys.into_iter();
    move || keys.next()
}

fn open(path: &Path) -> (LedgerFile, Ledger) {
    LedgerFile::open(path).unwrap()
}

fn browsed(path: &Path, keys: Vec<Key>) -> Screen {
    let (mut file, ledger) = open(path);
    browse(
        &mut file,
        Screen::new(ledger),
        &mut script(keys),
        &mut Vec::new(),
        &unknown,
    )
}

fn shown(path: &Path, keys: Vec<Key>) -> (u8, String) {
    let mut err = Vec::new();
    let code = show(
        path,
        || Ok(()),
        &mut script(keys),
        Vec::new(),
        &mut err,
        unknown,
    );
    (code, String::from_utf8(err).unwrap())
}

#[test]
fn toggling_writes_back_and_reads_back() {
    let path = file_with("toggle", FIXTURE);
    let keys = vec![Key::Down, Key::Char(' '), Key::Char('q')];
    assert_eq!(shown(&path, keys), (0, String::new()));
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(
        text,
        ledger::set_selection(FIXTURE, number(2), true).unwrap()
    );
    assert_eq!(ledger::read(&text).unwrap().selected(), vec![number(2)]);
}

#[test]
fn an_edit_outside_is_told_and_not_overwritten() {
    let path = file_with("outside", FIXTURE);
    let edited = format!("{FIXTURE}\n外で足した行\n");
    let mut sent = 0;
    let target = path.clone();
    let mut keys = move || {
        sent += 1;
        let key = [Key::Char(' '), Key::Char('q')].get(sent - 1).copied();
        (sent == 1).then(|| fs::write(&target, &edited).unwrap());
        key
    };
    let (mut file, ledger) = open(&path);
    let screen = browse(
        &mut file,
        Screen::new(ledger),
        &mut keys,
        &mut Vec::new(),
        &unknown,
    );
    assert_eq!(screen.notice, Some(Notice::ChangedOutside));
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        format!("{FIXTURE}\n外で足した行\n")
    );
}

#[test]
fn a_vanished_file_is_told_as_a_failed_write() {
    let path = file_with("vanished", FIXTURE);
    let (mut file, ledger) = open(&path);
    fs::remove_file(&path).unwrap();
    let keys = vec![Key::Char(' ')];
    let screen = browse(
        &mut file,
        Screen::new(ledger),
        &mut script(keys),
        &mut Vec::new(),
        &unknown,
    );
    assert_eq!(screen.notice, Some(Notice::WriteFailed));
}

/// The filter, cursor and whether an issue is open, after `keys`.
fn at(path: &Path, keys: Vec<Key>) -> (Option<Kind>, usize, bool) {
    let screen = browsed(path, keys);
    (screen.filter, screen.cursor, screen.open)
}

#[test]
fn keys_filter_and_open() {
    let path = file_with("filter", FIXTURE);
    let visibility = Some(Kind::Visibility);
    let keys = vec![Key::Char('2'), Key::Enter];
    assert_eq!(at(&path, keys), (visibility, 0, true));
    let keys = vec![Key::Char('2'), Key::Enter, Key::Char('0')];
    assert_eq!(at(&path, keys), (None, 0, false));
}

#[test]
fn keys_move_and_leave() {
    let path = file_with("move", FIXTURE);
    assert_eq!(
        at(&path, vec![Key::Down, Key::Down, Key::Down]),
        (None, 2, false)
    );
    assert_eq!(
        at(&path, vec![Key::Down, Key::Up, Key::Up]),
        (None, 0, false)
    );
    assert_eq!(
        at(&path, vec![Key::Other, Key::Ctrl('c'), Key::Down]),
        (None, 0, false)
    );
}

#[test]
fn toggling_with_nothing_shown_writes_nothing() {
    let path = file_with("nothing", FIXTURE);
    let keys = vec![Key::Char('4'), Key::Char(' ')];
    assert_eq!(browsed(&path, keys).notice, None);
    assert_eq!(fs::read_to_string(&path).unwrap(), FIXTURE);
}

#[test]
fn a_ledger_that_cannot_be_opened_exits_two_even_without_a_terminal() {
    let missing = temp_dir().join(format!("webqa-ledger-missing-{}", id()));
    let broken = file_with("broken", "台帳ではない");
    let not_a_ledger = ledger::read("台帳ではない").unwrap_err();
    let cases = [
        (missing, FileError::Io(ErrorKind::NotFound)),
        (broken, FileError::Ledger(not_a_ledger)),
    ];
    for (path, error) in cases {
        let mut err = Vec::new();
        let no_terminal = || Err::<(), _>(io::Error::from(ErrorKind::Unsupported));
        let keys = &mut script(vec![]);
        let code = show(&path, no_terminal, keys, Vec::new(), &mut err, unknown);
        assert_eq!(code, 2);
        assert_eq!(String::from_utf8(err).unwrap(), format!("{error}\n"));
    }
}

#[test]
fn without_a_terminal_the_screen_does_not_open() {
    let path = file_with("no-terminal", FIXTURE);
    let mut err = Vec::new();
    let no_terminal = || Err::<(), _>(io::Error::from(ErrorKind::Unsupported));
    let code = show(
        &path,
        no_terminal,
        &mut script(vec![]),
        Vec::new(),
        &mut err,
        unknown,
    );
    assert_eq!(code, 3);
    assert_eq!(String::from_utf8(err).unwrap(), format!("{NO_TERMINAL}\n"));
}
