//! The Given steps of the re-run: the run before, whose ledger is handed
//! over.

use std::mem;

use cucumber::given;
use webqa_core::config::purpose::{Aim, Written};
use webqa_core::config::raw::DoneWhen;
use webqa_core::interview::issue::Kind;
use webqa_core::interview::ledger::set_selection;

use super::course::{GUESS, Spot, at, course, number, spot, stash, trials_at};
use crate::ledger::read_back;
use crate::page::PageSpec;
use crate::scene::found;
use crate::world::InterviewWorld;

/// Runs the scene as the run before, and hands its ledger to the next run.
async fn hand_over(world: &mut InterviewWorld) {
    world.run().await;
    world.again();
    let text = world.before().store.ledger.clone();
    assert!(text.is_some(), "the run before wrote a ledger");
    world.scene.previous = text;
}

/// The run before: `reached` of `total` trials hesitate on `at`, and the
/// ledger has the issue.
pub(super) async fn issue_before(world: &mut InterviewWorld, at: Spot, total: u32, reached: u32) {
    course(
        &mut world.scene,
        &trials_at(total, reached, reached, at),
        GUESS,
    );
    let number = number(at, Kind::Hesitant);
    world.scene.aliases.bind(at.issue, number);
    hand_over(world).await;
    let ledger = read_back(found(world.scene.previous.as_deref(), "handed over"));
    let has = ledger.entries.iter().any(|e| e.issue.number == number);
    assert!(has, "the ledger before has the issue of {}", at.issue);
}

/// The screen of the number `issue` of the feature, which now stands for
/// the hesitation on it.
pub(super) fn bound(world: &mut InterviewWorld, issue: String) -> Spot {
    let at = spot(&issue);
    world.scene.aliases.bind(&issue, number(at, Kind::Hesitant));
    drop(issue);
    at
}

/// A written purpose with the goal `goal`.
fn written(goal: &str) -> Aim {
    let done_when = DoneWhen {
        url: Some("/order/confirm".to_owned()),
        element: None,
        text: None,
    };
    Aim::Written(Written {
        goal: goal.to_owned(),
        done_when,
    })
}

/// The run before, of the issue on the first screen.
async fn run_before(world: &mut InterviewWorld) {
    course(&mut world.scene, &trials_at(3, 3, 3, spot("I-0001")), GUESS);
    hand_over(world).await;
}

#[given(expr = "前回の台帳に課題 {string} がある")]
async fn before_has(world: &mut InterviewWorld, issue: String) {
    issue_before(world, spot(issue), 3, 3).await;
}

#[given(expr = "前回の台帳に課題 {string} が無い")]
async fn before_lacks(world: &mut InterviewWorld, issue: String) {
    let other = spot("I-0001");
    issue_before(world, other, 3, 3).await;
    let new = number(bound(world, issue), Kind::Hesitant);
    let ledger = read_back(found(world.scene.previous.as_deref(), "handed over"));
    assert!(
        !ledger.entries.iter().any(|e| e.issue.number == new),
        "no such issue"
    );
}

#[given(
    expr = "前回の台帳に、{int} 回の試行のうち {int} 回が画面 {string} に届いて出た課題 {string} がある"
)]
async fn before_reached(
    world: &mut InterviewWorld,
    total: u32,
    reached: u32,
    screen: String,
    issue: String,
) {
    issue_before(world, at(issue, screen), total, reached).await;
}

#[given(expr = "前回の台帳に、3回の試行のうち2回が画面 {string} に届いて出た課題 {string} がある")]
async fn before_two_of_three(world: &mut InterviewWorld, screen: String, issue: String) {
    issue_before(world, at(issue, screen), 3, 2).await;
}

#[given(expr = "前回の台帳に、画面 {string} に出た課題 {string} がある")]
async fn before_on_screen(world: &mut InterviewWorld, screen: String, issue: String) {
    issue_before(world, at(issue, screen), 3, 3).await;
}

#[given(expr = "前回の台帳で課題 {string} を選んでいる")]
async fn before_selected(world: &mut InterviewWorld, issue: String) {
    issue_before(world, spot(&issue), 3, 3).await;
    let number = world.scene.aliases.number(&issue);
    let text = found(world.scene.previous.as_deref(), "handed over");
    let selected = set_selection(text, number, true);
    world.scene.previous = Some(found(selected.ok(), "the checkbox is set"));
}

#[given(expr = "前回の台帳を渡していない")]
fn before_none(world: &mut InterviewWorld) {
    let handed_over = world.scene.previous.take();
    assert_eq!(handed_over, None, "no ledger is handed over");
}

#[given(expr = "前回の台帳として渡したファイルの課題の区切りが崩れていて、読み戻せない")]
async fn before_broken(world: &mut InterviewWorld) {
    issue_before(world, spot("I-0001"), 3, 3).await;
    let text = found(world.scene.previous.as_deref(), "handed over");
    let broken = text.replace("\n- [ ] I-", "\n* [ ] I-");
    assert_ne!(broken, text, "the separators are broken");
    world.scene.previous = Some(broken);
}

#[given(expr = "前回の台帳の ペルソナの名前 が今回の設定と違う")]
async fn before_other_persona(world: &mut InterviewWorld) {
    let name = mem::replace(&mut world.scene.persona.name, "田中さん".to_owned());
    run_before(world).await;
    world.scene.persona.name = name;
}

#[given(expr = "前回の台帳の 書いた目的 が今回の設定と違う")]
async fn before_other_purpose(world: &mut InterviewWorld) {
    world.scene.purpose.aim = written("前回の目的");
    run_before(world).await;
    world.scene.purpose.aim = written("今回の目的");
}

#[given(expr = "前回の台帳の 目的を推測させた実行どうしの開始の URL が今回の設定と違う")]
async fn before_other_start(world: &mut InterviewWorld) {
    let start = mem::replace(&mut world.scene.start, PageSpec::at("/shop"));
    run_before(world).await;
    world.scene.start = start;
}

#[given(expr = "前回の台帳は、目的を推測させた実行のもので、推測した文は {string} である")]
async fn before_guessed(world: &mut InterviewWorld, guess: String) {
    let at = spot("I-0001");
    course(&mut world.scene, &trials_at(3, 3, 3, at), &guess);
    world
        .scene
        .aliases
        .bind(at.issue, number(at, Kind::Hesitant));
    stash(world, "GUESS_BEFORE", &guess);
    hand_over(world).await;
    drop(guess);
}
