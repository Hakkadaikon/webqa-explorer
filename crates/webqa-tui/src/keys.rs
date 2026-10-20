//! The keys of the ledger screen, and the real terminal that gives them.

use std::convert::identity;
use std::io::{self, ErrorKind, IsTerminal, stdin};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use webqa_core::interview::issue::Kind;

use crate::ledger::Input;

/// A key the screen cares about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// The up arrow.
    Up,
    /// The down arrow.
    Down,
    /// The left arrow.
    Left,
    /// The right arrow.
    Right,
    /// Enter.
    Enter,
    /// Escape.
    Esc,
    /// A character.
    Char(char),
    /// A character with Ctrl held.
    Ctrl(char),
    /// Anything else: a resize, a release, a key with no meaning.
    Other,
}

/// What a key asks of the screen.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Fold this input.
    Fold(Input),
    /// Toggle the selection of the issue under the cursor.
    Toggle,
    /// Leave the screen.
    Quit,
    /// Nothing.
    Ignore,
}

/// Where keys come from. `None` is the end of input.
pub trait Keys {
    /// Waits for the next key.
    fn next(&mut self) -> Option<Key>;
}

impl<F: FnMut() -> Option<Key>> Keys for F {
    fn next(&mut self) -> Option<Key> {
        self()
    }
}

/// What each key asks, apart from the ones that are no command.
static BINDINGS: [(Key, Command); 18] = [
    (Key::Up, Command::Fold(Input::Up)),
    (Key::Char('k'), Command::Fold(Input::Up)),
    (Key::Down, Command::Fold(Input::Down)),
    (Key::Char('j'), Command::Fold(Input::Down)),
    (Key::Enter, Command::Fold(Input::Open)),
    (Key::Right, Command::Fold(Input::Open)),
    (Key::Char('l'), Command::Fold(Input::Open)),
    (Key::Esc, Command::Fold(Input::Close)),
    (Key::Left, Command::Fold(Input::Close)),
    (Key::Char('h'), Command::Fold(Input::Close)),
    (Key::Char(' '), Command::Toggle),
    (
        Key::Char('1'),
        Command::Fold(Input::Filter(Some(Kind::Hesitant))),
    ),
    (
        Key::Char('2'),
        Command::Fold(Input::Filter(Some(Kind::Visibility))),
    ),
    (
        Key::Char('3'),
        Command::Fold(Input::Filter(Some(Kind::Unreached))),
    ),
    (
        Key::Char('4'),
        Command::Fold(Input::Filter(Some(Kind::Mismatch))),
    ),
    (Key::Char('0'), Command::Fold(Input::Filter(None))),
    (Key::Char('q'), Command::Quit),
    (Key::Ctrl('c'), Command::Quit),
];

/// The keys that are not characters.
const NAMED: [(KeyCode, Key); 6] = [
    (KeyCode::Up, Key::Up),
    (KeyCode::Down, Key::Down),
    (KeyCode::Left, Key::Left),
    (KeyCode::Right, Key::Right),
    (KeyCode::Enter, Key::Enter),
    (KeyCode::Esc, Key::Esc),
];

/// What `key` asks.
#[must_use]
pub fn command(key: Key) -> Command {
    let bound = BINDINGS.iter().find(|(bound, _)| *bound == key);
    bound.map_or(Command::Ignore, |(_, command)| command.clone())
}

/// The key of a terminal event. Only presses count.
#[must_use]
pub fn key_of(event: &Event) -> Key {
    match event {
        Event::Key(key) if key.kind != KeyEventKind::Release => {
            code_key(key.code, key.modifiers.contains(KeyModifiers::CONTROL))
        }
        Event::Key(_)
        | Event::FocusGained
        | Event::FocusLost
        | Event::Mouse(_)
        | Event::Paste(_)
        | Event::Resize(..) => Key::Other,
    }
}

fn code_key(code: KeyCode, ctrl: bool) -> Key {
    if let KeyCode::Char(c) = code {
        return char_key(c, ctrl);
    }
    let named = NAMED.iter().find(|(named, _)| *named == code);
    named.map_or(Key::Other, |(_, key)| *key)
}

const fn char_key(c: char, ctrl: bool) -> Key {
    if ctrl { Key::Ctrl(c) } else { Key::Char(c) }
}

/// The keys of events from `read`. An event that cannot be read ends the
/// input.
pub fn events(mut read: impl FnMut() -> io::Result<Event>) -> impl Keys {
    move || read().ok().as_ref().map(key_of)
}

/// The keys of the real terminal.
pub fn terminal() -> impl Keys {
    events(event::read)
}

/// The terminal in raw mode; put back when dropped, by the function it holds.
#[derive(Debug)]
pub struct RawMode(pub fn() -> io::Result<()>);

impl RawMode {
    // No branch on purpose: a test cannot reach a real terminal, so the
    // success side must not be a line of its own that no test runs.
    /// Enters raw mode. Only a terminal on the standard input can be read,
    /// so any other input is refused.
    ///
    /// # Errors
    ///
    /// When the standard input is not a terminal, or raw mode fails.
    pub fn enter() -> io::Result<Self> {
        stdin()
            .is_terminal()
            .then(enable_raw_mode)
            .ok_or(ErrorKind::Unsupported)
            .map_err(io::Error::from)
            .and_then(identity)
            .map(Self::from)
    }
}

impl From<()> for RawMode {
    fn from((): ()) -> Self {
        Self(disable_raw_mode)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        drop((self.0)());
    }
}

#[cfg(test)]
mod tests;
