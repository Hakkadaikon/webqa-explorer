//! The ledger screen of the scenarios: a ledger file in a directory of its
//! own, the keys of the user (and of an editor that saves the file once the
//! screen is open), and the frames the screen drew.

use std::collections::VecDeque;
use std::env::temp_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::id;
use std::sync::atomic::{AtomicUsize, Ordering};

use webqa::ledger::show;
use webqa_core::interview::issue::Number;
use webqa_core::interview::ledger::Ledger;
use webqa_tui::keys::{Key, Keys};

use super::frame::frame_of;
use crate::ledger::read_back;
use crate::rerun::course::{stash, stashed};
use crate::scene::found;
use crate::world::InterviewWorld;

/// How many directories were made by this process.
static DIRS: AtomicUsize = AtomicUsize::new(0);

/// The ledger the user has open.
const LEDGER: &str = "ledger.md";
/// The ledger as it was when the screen was opened.
const OPENED: &str = "opened.md";
/// The ledger an editor saves once the screen is open.
const EDITED: &str = "edited.md";
/// What the screen drew.
const SCREEN: &str = "screen.txt";
/// The key the directory is stashed under.
const DIR: &str = "TUI_DIR";

/// A new directory for the scenario.
pub fn make_dir(world: &mut InterviewWorld) {
    let n = DIRS.fetch_add(1, Ordering::SeqCst);
    let dir = temp_dir().join(format!("webqa-tui-{}-{n}", id()));
    let _ = fs::remove_dir_all(&dir);
    assert!(fs::create_dir_all(&dir).is_ok(), "the directory is made");
    stash(world, DIR, &dir.to_string_lossy());
}

/// The ledger file in a new directory, and the same text as it was opened.
pub fn open(world: &mut InterviewWorld, text: &str) {
    make_dir(world);
    let dir = dir(world);
    write(&dir.join(LEDGER), text);
    write(&dir.join(OPENED), text);
}

fn write(path: &Path, text: &str) {
    assert!(fs::write(path, text).is_ok(), "the file is written");
}

fn read(path: &Path) -> String {
    let text = fs::read_to_string(path);
    assert!(text.is_ok(), "the file reads");
    text.unwrap_or_default()
}

fn dir(world: &mut InterviewWorld) -> PathBuf {
    PathBuf::from(stashed(world, DIR))
}

/// The ledger as it was when the screen was opened.
pub fn opened(world: &mut InterviewWorld) -> String {
    read(&dir(world).join(OPENED))
}

/// The ledger file, as it is now.
pub fn now(world: &mut InterviewWorld) -> String {
    read(&dir(world).join(LEDGER))
}

/// Saves `text` as the ledger once the screen is open.
pub fn edit_later(world: &mut InterviewWorld, text: &str) {
    write(&dir(world).join(EDITED), text);
}

/// The keys of a user; the editor saves before the first key is read.
struct Typed {
    keys: VecDeque<Key>,
    dir: PathBuf,
    started: bool,
}

impl Keys for Typed {
    fn next(&mut self) -> Option<Key> {
        let edited = self.dir.join(EDITED);
        if !self.started && edited.is_file() {
            assert!(
                fs::rename(&edited, self.dir.join(LEDGER)).is_ok(),
                "the editor saves"
            );
        }
        self.started = true;
        self.keys.pop_front()
    }
}

/// The keys that move the cursor to the issue `number` of the ledger.
pub fn down_to(ledger: &Ledger, number: Number) -> Vec<Key> {
    let at = ledger.entries.iter().position(|e| e.issue.number == number);
    vec![Key::Down; found(at, "the issue is in the ledger")]
}

/// Opens the ledger screen on the file, types `keys`, and keeps what was
/// drawn. The terminal is none; the screen draws to memory.
pub fn type_keys(world: &mut InterviewWorld, keys: Vec<Key>) {
    let dir = dir(world);
    let mut typed = Typed {
        keys: keys.into(),
        dir: dir.clone(),
        started: false,
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = show(&dir.join(LEDGER), || Ok(()), &mut typed, &mut out, &mut err);
    assert_eq!((code, err), (0, Vec::new()), "the screen ended well");
    write(&dir.join(SCREEN), &String::from_utf8_lossy(&out));
}

/// The last thing the screen drew, as the text on it without its blanks. A
/// wide character is drawn with a move of the cursor around it, and a blank
/// is not drawn at all, so the escapes and the blanks are cut out.
pub fn last_frame(world: &mut InterviewWorld) -> String {
    frame_of(&read(&dir(world).join(SCREEN)))
}

/// The log of the run in the directory of the scenario.
pub fn log(world: &mut InterviewWorld) -> PathBuf {
    dir(world).join("events.jsonl")
}

/// The directory of the scenario.
pub fn work_dir(world: &mut InterviewWorld) -> PathBuf {
    dir(world)
}

/// What the viewer drew, kept for the Then.
pub fn keep_screen(world: &mut InterviewWorld, out: &[u8]) {
    write(&dir(world).join(SCREEN), &String::from_utf8_lossy(out));
}

/// The ledger file, as it is now, read back.
pub fn ledger_now(world: &mut InterviewWorld) -> Ledger {
    read_back(&now(world))
}
