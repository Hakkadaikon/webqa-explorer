//! The Then steps of the ledger screen: the file, and what the screen drew.

use cucumber::then;
use webqa_core::interview::issue::Kind;
use webqa_core::interview::ledger::set_selection;
use webqa_tui::ledger::{Notice, notice_text};

use super::frame::{detail_lines, shows};
use super::tui;
use crate::rerun::course::{number, spot};
use crate::scene::found;
use crate::world::InterviewWorld;

#[then(expr = "同じ台帳のファイルで、{string} のチェックボックスが付いている")]
fn file_has_checked(world: &mut InterviewWorld, issue: String) {
    let selected = tui::ledger_now(world).selected();
    assert_eq!(
        selected,
        [world.scene.aliases.number(&issue)],
        "the selected issues"
    );
    drop(issue);
}

#[then(expr = "台帳のファイルの中身は、開く前と同じである")]
fn file_is_as_opened(world: &mut InterviewWorld) {
    assert_eq!(tui::now(world), tui::opened(world), "the file");
}

#[then(expr = "台帳のファイルは上書きされておらず、{string} のチェックボックスは付いたままである")]
fn file_kept_the_edit(world: &mut InterviewWorld, issue: String) {
    let number = world.scene.aliases.number(&issue);
    let edited = set_selection(&tui::opened(world), number, true);
    assert_eq!(
        tui::now(world),
        found(edited.ok(), "the checkbox is set"),
        "the file"
    );
    assert_eq!(
        tui::ledger_now(world).selected(),
        [number],
        "the selected issues"
    );
    drop(issue);
}

#[then(expr = "TUI に、台帳が外で書き換えられたことが表示されている")]
fn screen_says_changed(world: &mut InterviewWorld) {
    let frame = tui::last_frame(world);
    assert!(
        shows(&frame, notice_text(Notice::ChangedOutside)),
        "the notice is drawn"
    );
}

#[then(expr = "一覧には視認性が悪い課題1件だけが出ている")]
fn only_visibility(world: &mut InterviewWorld) {
    let frame = tui::last_frame(world);
    let (hesitant, faint) = (Kind::Hesitant, Kind::Visibility);
    let shown = |issue: &str, kind| number(spot(issue), kind).to_string();
    let rows = frame.matches("]I-").count();
    let title = format!("台帳 ({})", faint.label());
    let drawn = (
        rows,
        shows(&frame, &title),
        frame.contains(&shown("I-0003", faint)),
        frame.contains(&shown("I-0001", hesitant)),
        frame.contains(&shown("I-0002", hesitant)),
    );
    assert_eq!(
        drawn,
        (1, true, true, false, false),
        "rows, title, and the issues drawn"
    );
}

#[then(expr = "{string} の直す指示の文、操作の列、ペルソナの性質、理由と期待が表示されている")]
fn issue_is_read(world: &mut InterviewWorld, issue: String) {
    let number = world.scene.aliases.number(&issue);
    let ledger = tui::ledger_now(world);
    let entry = found(
        ledger.entries.iter().find(|e| e.issue.number == number),
        "the issue",
    );
    let frame = tui::last_frame(world);
    let lines = found(detail_lines(entry), "a hesitation");
    let lacking: Vec<&String> = lines.iter().filter(|line| !shows(&frame, line)).collect();
    assert_eq!(
        lacking,
        Vec::<&String>::new(),
        "the lines of the issue not drawn"
    );
    assert!(
        frame.contains(&number.to_string()),
        "the number is the title"
    );
    drop(issue);
}
