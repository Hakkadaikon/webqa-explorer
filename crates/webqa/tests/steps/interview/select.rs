//! Steps of this Rule of the feature, and the scenarios that have them.
//! The ledger screen is the one of `webqa ledger`, with keys typed from a
//! script and no terminal.

use cucumber::{given, then, when};
use webqa_core::interview::issue::Kind;
use webqa_tui::keys::Key;

use crate::rerun::course::{GUESS, Mark, course, number, spot};
use crate::scene::found;
use crate::world::InterviewWorld;

#[path = "select_frame.rs"]
mod frame;
#[path = "select_then.rs"]
mod then;
#[path = "select_tui.rs"]
mod tui;
#[path = "select_watch.rs"]
mod watch;

/// The scenarios of this Rule that have their steps.
pub const READY: &[&str] = &[
    "Markdown のチェックボックスを直接書き換えて選んだ課題は、選んだ課題として読み戻せる",
    "TUI で課題の選択を付け外しすると、同じ Markdown に書き戻す",
    "TUI で書き戻しても、選択のほかの中身は変えない",
    "TUI で開いたあとに台帳が外で書き換えられていたら、上書きせずに知らせる",
    "TUI で課題を種類で絞り込める",
    "TUI で課題を選ぶと、その中身を読める",
    "実行中のイベントログを追う画面は、今のまま使える",
];

/// Runs trials that hesitate on the three screens of `I-0001` to `I-0003`
/// in turn, and gives the ledger of the run.
async fn finished_run(world: &mut InterviewWorld, marks: [Mark; 3]) -> String {
    let issues = ["I-0001", "I-0002", "I-0003"];
    let walk: Vec<_> = issues
        .iter()
        .zip(marks)
        .map(|(issue, mark)| (spot(issue), mark))
        .collect();
    for (issue, (at, mark)) in issues.iter().zip(&walk) {
        let kind = if *mark == Mark::Faint {
            Kind::Visibility
        } else {
            Kind::Hesitant
        };
        world.scene.aliases.bind(issue, number(*at, kind));
    }
    course(&mut world.scene, &[walk.clone(), walk.clone(), walk], GUESS);
    world.run().await;
    found(world.ran().store.ledger.clone(), "the run wrote a ledger")
}

/// The keys that put the cursor on `issue`.
fn cursor_on(world: &mut InterviewWorld, issue: &str) -> Vec<Key> {
    tui::down_to(&tui::ledger_now(world), world.scene.aliases.number(issue))
}

#[given(expr = "台帳の課題 {string} のチェックボックスを、エディタで付ける")]
async fn checked_in_editor(world: &mut InterviewWorld, issue: String) {
    finished_run(world, [Mark::Hesitant; 3]).await;
    let text = found(world.ran().store.ledger.clone(), "a ledger");
    let number = world.scene.aliases.number(&issue);
    let edited = tui::ticked(&text, number);
    let ran = found(world.ran.as_mut(), "the scene ran");
    ran.store.ledger = Some(edited);
    drop(issue);
}

#[then(expr = "選んだ課題は {string} だけである")]
async fn only_selected(world: &mut InterviewWorld, issue: String) {
    world.run().await;
    let selected = found(world.ledger(), "the ledger reads back").selected();
    assert_eq!(
        selected,
        [world.scene.aliases.number(&issue)],
        "the selected issues"
    );
    drop(issue);
}

#[given(expr = "終わった実行の台帳を TUI で開いている")]
#[given(expr = "台帳を TUI で開いている")]
async fn opened(world: &mut InterviewWorld) {
    let text = finished_run(world, [Mark::Hesitant; 3]).await;
    tui::open(world, &text);
}

#[given(expr = "判断に迷う課題が2件、視認性が悪い課題が1件ある台帳を TUI で開いている")]
async fn opened_with_two_kinds(world: &mut InterviewWorld) {
    let text = finished_run(world, [Mark::Hesitant, Mark::Hesitant, Mark::Faint]).await;
    tui::open(world, &text);
}

#[given(expr = "そのあとに、エディタで台帳の課題 {string} のチェックボックスを付けて保存している")]
fn edited_after(world: &mut InterviewWorld, issue: String) {
    let number = world.scene.aliases.number(&issue);
    let edited = tui::ticked(&tui::opened(world), number);
    tui::edit_later(world, &edited);
    drop(issue);
}

#[when(expr = "課題 {string} の選択を付ける")]
#[when(expr = "TUI で課題 {string} の選択を付ける")]
fn select(world: &mut InterviewWorld, issue: String) {
    let keys = [
        cursor_on(world, &issue),
        vec![Key::Char(' '), Key::Char('q')],
    ]
    .concat();
    tui::type_keys(world, keys);
    drop(issue);
}

#[when(expr = "課題 {string} の選択を付け、また外す")]
fn select_and_clear(world: &mut InterviewWorld, issue: String) {
    let toggles = vec![Key::Char(' '), Key::Char(' '), Key::Char('q')];
    let keys = [cursor_on(world, &issue), toggles].concat();
    tui::type_keys(world, keys);
    drop(issue);
}

#[when(expr = "種類を視認性が悪いに絞る")]
fn filter_by_visibility(world: &mut InterviewWorld) {
    tui::type_keys(world, vec![Key::Char('2'), Key::Char('q')]);
}

#[when(expr = "一覧から課題 {string} を開く")]
fn open_issue(world: &mut InterviewWorld, issue: String) {
    let keys = [cursor_on(world, &issue), vec![Key::Enter, Key::Char('q')]].concat();
    tui::type_keys(world, keys);
    drop(issue);
}
