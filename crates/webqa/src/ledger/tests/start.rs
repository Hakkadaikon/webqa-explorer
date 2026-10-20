//! Tests of the ledger screen that does not open.

use super::*;

/// No terminal.
fn no_terminal() -> io::Result<RawMode> {
    Err(io::Error::from(ErrorKind::Unsupported))
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
        let keys = &mut script(vec![]);
        let code = show(
            &path,
            &no_terminal,
            keys,
            &mut Vec::new(),
            &mut err,
            &unknown,
        );
        assert_eq!(code, 2);
        assert_eq!(String::from_utf8(err).unwrap(), format!("{error}\n"));
    }
}

#[test]
fn without_a_terminal_the_screen_does_not_open() {
    let path = file_with("no-terminal", FIXTURE);
    let mut err = Vec::new();
    let code = show(
        &path,
        &no_terminal,
        &mut script(vec![]),
        &mut Vec::new(),
        &mut err,
        &unknown,
    );
    assert_eq!(code, 3);
    assert_eq!(String::from_utf8(err).unwrap(), format!("{NO_TERMINAL}\n"));
}
