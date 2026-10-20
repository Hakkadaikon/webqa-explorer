//! Tests of what the screen draws, and of putting the terminal back.

use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

use super::*;

/// Sets the mark when dropped, as the terminal is put back.
struct Mark<'a>(&'a Cell<bool>);

impl Drop for Mark<'_> {
    fn drop(&mut self) {
        self.0.set(true);
    }
}

/// A writer that cannot write.
struct Closed;

impl Write for Closed {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::from(ErrorKind::BrokenPipe))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::from(ErrorKind::BrokenPipe))
    }
}

#[test]
fn the_screen_is_drawn_to_the_output() {
    let path = file_with("drawn", FIXTURE);
    let mut out = Vec::new();
    let keys = &mut script(vec![Key::Char('q')]);
    assert_eq!(
        show(&path, || Ok(()), keys, &mut out, Vec::new(), unknown),
        0
    );
    assert!(String::from_utf8_lossy(&out).contains("I-00000001"));
}

#[test]
fn keys_still_count_when_the_screen_cannot_be_drawn() {
    let path = file_with("closed", FIXTURE);
    let keys = &mut script(vec![Key::Char(' ')]);
    assert_eq!(show(&path, || Ok(()), keys, Closed, Vec::new(), unknown), 0);
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(ledger::read(&text).unwrap().selected(), vec![number(1)]);
}

#[test]
fn the_terminal_is_put_back_when_the_screen_closes() {
    let path = file_with("put-back", FIXTURE);
    let put_back = Cell::new(false);
    let keys = &mut script(vec![]);
    show(
        &path,
        || Ok(Mark(&put_back)),
        keys,
        Vec::new(),
        Vec::new(),
        unknown,
    );
    assert!(put_back.get());
}

#[test]
fn the_terminal_is_put_back_when_the_keys_panic() {
    let path = file_with("panic", FIXTURE);
    let put_back = Cell::new(false);
    let mut keys = || resume_unwind(Box::new(()));
    let run = AssertUnwindSafe(|| {
        show(
            &path,
            || Ok(Mark(&put_back)),
            &mut keys,
            Vec::new(),
            Vec::new(),
            unknown,
        )
    });
    assert!(catch_unwind(run).is_err());
    assert!(put_back.get());
}

#[test]
fn the_screen_is_drawn_in_the_size_the_terminal_has() {
    let path = file_with("size", FIXTURE);
    let keys = &mut script(vec![Key::Enter, Key::Char('q')]);
    let mut drawn = Vec::new();
    let size = || Some((30, 24));
    let code = show(&path, || Ok(()), keys, &mut drawn, Vec::new(), size);
    let list = Screen::new(ledger::read(FIXTURE).unwrap());
    let mut expected = Vec::new();
    draw(&mut expected, &list, size()).unwrap();
    draw(&mut expected, &list.fold(Input::Open), size()).unwrap();
    assert_eq!((code, drawn), (0, expected));
}
