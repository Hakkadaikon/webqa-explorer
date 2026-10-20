//! `webqa ledger`: opens the ledger of an interview, to choose issues in.

#[cfg(test)]
mod tests;

use std::io::{self, Write};
use std::path::Path;

use webqa_tui::keys::{Command, Keys, command};
use webqa_tui::ledger::render::draw;
use webqa_tui::ledger::{Input, Screen};
use webqa_tui::ledger_file::{FileError, LedgerFile};

use crate::exit::{CONFIG_ERROR, RUNTIME_ERROR};

/// What is said when there is no terminal to draw the screen on.
const NO_TERMINAL: &str = "端末が無いので台帳の画面を開けない";

/// Opens the ledger at `path` and runs the screen until the user leaves;
/// the exit code. `enter` takes the terminal and gives what puts it back
/// when dropped. A ledger that cannot be opened is checked first.
pub fn show<G>(
    path: &Path,
    enter: impl FnOnce() -> io::Result<G>,
    keys: &mut dyn Keys,
    mut out: impl Write,
    mut err: impl Write,
) -> u8 {
    let (mut file, ledger) = match LedgerFile::open(path) {
        Ok(opened) => opened,
        Err(error) => {
            drop(writeln!(err, "{error}"));
            return CONFIG_ERROR;
        }
    };
    let Ok(terminal) = enter() else {
        drop(writeln!(err, "{NO_TERMINAL}"));
        return RUNTIME_ERROR;
    };
    browse(&mut file, Screen::new(ledger), keys, &mut out);
    drop(terminal);
    0
}

/// Draws and reads keys until the user leaves or the keys end; the last
/// screen. A screen that cannot be drawn is not an error: the keys still count.
fn browse(
    file: &mut LedgerFile,
    mut screen: Screen,
    keys: &mut dyn Keys,
    out: &mut impl Write,
) -> Screen {
    loop {
        drop(draw(out, &screen));
        let asked = keys.next().map_or(Command::Quit, command);
        if asked == Command::Quit {
            return screen;
        }
        screen = answered(file, screen, asked);
    }
}

/// `screen` after `asked`.
fn answered(file: &mut LedgerFile, screen: Screen, asked: Command) -> Screen {
    match asked {
        Command::Toggle => toggled(file, screen),
        Command::Fold(input) => screen.fold(input),
        Command::Ignore | Command::Quit => screen,
    }
}

/// `screen` after toggling the issue under the cursor and writing it back.
fn toggled(file: &mut LedgerFile, screen: Screen) -> Screen {
    let Some((number, selected)) = screen.toggle() else {
        return screen;
    };
    let input = match file.set_selection(number, selected) {
        Ok(ledger) => Input::Saved(ledger),
        Err(FileError::ChangedOutside) => Input::ChangedOutside,
        Err(FileError::Io(_) | FileError::Ledger(_)) => Input::WriteFailed,
    };
    screen.fold(input)
}
