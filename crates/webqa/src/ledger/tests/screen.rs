//! Tests of what the screen draws, and of putting the terminal back.

use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

use std::sync::atomic::{AtomicBool, Ordering};

use super::*;

/// Set when the terminal is put back after the screen closes.
static PUT_BACK: AtomicBool = AtomicBool::new(false);

/// Set when the terminal is put back while the keys panic.
static PUT_BACK_ON_PANIC: AtomicBool = AtomicBool::new(false);

fn marks(mark: &'static AtomicBool) -> io::Result<()> {
    mark.store(true, Ordering::SeqCst);
    // The result of putting back is not read, so a failure is as good.
    Err(io::Error::from(ErrorKind::Other))
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
        show(&path, NO_GUARD, keys, &mut out, &mut Vec::new(), &unknown),
        0
    );
    assert!(String::from_utf8_lossy(&out).contains("I-00000001"));
}

#[test]
fn keys_still_count_when_the_screen_cannot_be_drawn() {
    let path = file_with("closed", FIXTURE);
    let keys = &mut script(vec![Key::Char(' ')]);
    assert_eq!(
        show(
            &path,
            NO_GUARD,
            keys,
            &mut Closed,
            &mut Vec::new(),
            &unknown
        ),
        0
    );
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(ledger::read(&text).unwrap().selected(), vec![number(1)]);
}

#[test]
fn the_terminal_is_put_back_when_the_screen_closes() {
    let path = file_with("put-back", FIXTURE);
    let keys = &mut script(vec![]);
    let enter = || Ok(RawMode(|| marks(&PUT_BACK)));
    show(
        &path,
        &enter,
        keys,
        &mut Vec::new(),
        &mut Vec::new(),
        &unknown,
    );
    assert!(PUT_BACK.load(Ordering::SeqCst));
}

#[test]
fn the_terminal_is_put_back_when_the_keys_panic() {
    let path = file_with("panic", FIXTURE);
    let mut keys = || resume_unwind(Box::new(()));
    let run = AssertUnwindSafe(|| {
        show(
            &path,
            &|| Ok(RawMode(|| marks(&PUT_BACK_ON_PANIC))),
            &mut keys,
            &mut Vec::new(),
            &mut Vec::new(),
            &unknown,
        )
    });
    assert!(catch_unwind(run).is_err());
    assert!(PUT_BACK_ON_PANIC.load(Ordering::SeqCst));
}

#[test]
fn the_screen_is_drawn_in_the_size_the_terminal_has() {
    let path = file_with("size", FIXTURE);
    let keys = &mut script(vec![Key::Enter, Key::Char('q')]);
    let mut drawn = Vec::new();
    let size = || Some((30, 24));
    let code = show(&path, NO_GUARD, keys, &mut drawn, &mut Vec::new(), &size);
    let list = Screen::new(ledger::read(FIXTURE).unwrap());
    let mut expected = Vec::new();
    draw(&mut expected, &list, size()).unwrap();
    draw(&mut expected, &list.fold(Input::Open), size()).unwrap();
    assert_eq!((code, drawn), (0, expected));
}
