//! Tests of the keys.

use crossterm::event::{KeyEvent, KeyEventState};

use super::*;

fn press(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

#[test]
fn events_become_keys() {
    let none = KeyModifiers::NONE;
    let release = KeyEvent {
        code: KeyCode::Up,
        modifiers: none,
        kind: KeyEventKind::Release,
        state: KeyEventState::NONE,
    };
    let cases = [
        (press(KeyCode::Up, none), Key::Up),
        (press(KeyCode::Down, none), Key::Down),
        (press(KeyCode::Enter, none), Key::Enter),
        (press(KeyCode::Esc, none), Key::Esc),
        (press(KeyCode::Char('j'), none), Key::Char('j')),
        (
            press(KeyCode::Char('c'), KeyModifiers::CONTROL),
            Key::Ctrl('c'),
        ),
        (press(KeyCode::Tab, none), Key::Other),
        (Event::Key(release), Key::Other),
        (Event::Resize(1, 1), Key::Other),
    ];
    for (event, key) in cases {
        assert_eq!(key_of(&event), key, "{event:?}");
    }
}

fn asks(cases: &[(Key, Command)]) {
    for (key, want) in cases {
        assert_eq!(command(*key), *want, "{key:?}");
    }
}

#[test]
fn keys_move_open_and_toggle() {
    let fold = Command::Fold;
    asks(&[
        (Key::Up, fold(Input::Up)),
        (Key::Char('k'), fold(Input::Up)),
        (Key::Down, fold(Input::Down)),
        (Key::Char('j'), fold(Input::Down)),
        (Key::Enter, fold(Input::Open)),
        (Key::Esc, fold(Input::Close)),
        (Key::Char(' '), Command::Toggle),
    ]);
}

#[test]
fn keys_filter_quit_and_ignore() {
    let filter = |kind| Command::Fold(Input::Filter(kind));
    asks(&[
        (Key::Char('1'), filter(Some(Kind::Hesitant))),
        (Key::Char('2'), filter(Some(Kind::Visibility))),
        (Key::Char('3'), filter(Some(Kind::Unreached))),
        (Key::Char('4'), filter(Some(Kind::Mismatch))),
        (Key::Char('0'), filter(None)),
        (Key::Char('5'), Command::Ignore),
        (Key::Char('x'), Command::Ignore),
        (Key::Char('q'), Command::Quit),
        (Key::Ctrl('c'), Command::Quit),
        (Key::Ctrl('x'), Command::Ignore),
        (Key::Other, Command::Ignore),
    ]);
}

#[test]
fn a_closure_gives_keys() {
    let mut keys = || Some(Key::Esc);
    assert_eq!(Keys::next(&mut keys), Some(Key::Esc));
}

#[test]
fn events_become_keys_and_a_failed_read_ends_the_input() {
    let mut pressed = events(|| Ok(press(KeyCode::Enter, KeyModifiers::NONE)));
    assert_eq!(pressed.next(), Some(Key::Enter));
    let mut failed = events(|| Err(io::Error::from(ErrorKind::Unsupported)));
    assert_eq!(failed.next(), None);
}

#[test]
fn raw_mode_is_put_back_by_dropping() {
    drop(RawMode::from(()));
}
