//! The Given steps of the re-run: the run before, whose ledger is handed
//! over.

use std::mem;

use cucumber::given;
use webqa_core::config::purpose::{Aim, Written};
use webqa_core::config::raw::DoneWhen;
use webqa_core::interview::issue::Kind;

use super::course::{GUESS, Spot, at, course, number, spot, stash, ticked, trials_at};
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

/// How many trials the run before made, how many reached the screen, and
/// in how many of those the persona hesitated.
#[derive(Debug, Clone, Copy)]
pub(super) struct Before {
    pub(super) total: u32,
    pub(super) reached: u32,
    pub(super) occurred: u32,
}

/// The run before: the trials of `before` on `at`, and the ledger has the
/// issue with those counts.
pub(super) async fn issue_before(world: &mut InterviewWorld, at: Spot, before: Before) {
    let Before {
        total,
        reached,
        occurred,
    } = before;
    course(
        &mut world.scene,
        &trials_at(total, reached, occurred, at),
        GUESS,
    );
    let number = number(at, Kind::Hesitant);
    world.scene.aliases.bind(at.issue, number);
    hand_over(world).await;
    let ledger = read_back(found(world.scene.previous.as_deref(), "handed over"));
    let entry = ledger.entries.iter().find(|e| e.issue.number == number);
    let count = found(entry, "the ledger before has the issue").issue.count;
    let counts = (count.occurred, count.finished, count.reached);
    let size = |n: u32| found(usize::try_from(n).ok(), "a count fits");
    assert_eq!(
        counts,
        (size(occurred), size(total), size(reached)),
        "the counts of {}",
        at.issue
    );
}

/// Every trial reaches the screen and hesitates there.
pub(super) const ALL: Before = Before {
    total: 3,
    reached: 3,
    occurred: 3,
};

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
    issue_before(world, spot(issue), ALL).await;
}

#[given(expr = "前回の台帳に課題 {string} が無い")]
async fn before_lacks(world: &mut InterviewWorld, issue: String) {
    let other = spot("I-0001");
    issue_before(world, other, ALL).await;
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
    issue_before(
        world,
        at(issue, screen),
        Before {
            total,
            reached,
            occurred: 1,
        },
    )
    .await;
}

#[given(expr = "前回の台帳に、3回の試行のうち2回が画面 {string} に届いて出た課題 {string} がある")]
async fn before_two_of_three(world: &mut InterviewWorld, screen: String, issue: String) {
    issue_before(
        world,
        at(issue, screen),
        Before {
            total: 3,
            reached: 2,
            occurred: 1,
        },
    )
    .await;
}

#[given(expr = "前回の台帳に、画面 {string} に出た課題 {string} がある")]
async fn before_on_screen(world: &mut InterviewWorld, screen: String, issue: String) {
    issue_before(world, at(issue, screen), ALL).await;
}

#[given(expr = "前回の台帳で課題 {string} を選んでいる")]
async fn before_selected(world: &mut InterviewWorld, issue: String) {
    issue_before(world, spot(&issue), ALL).await;
    let number = world.scene.aliases.number(&issue);
    let text = found(world.scene.previous.as_deref(), "handed over");
    world.scene.previous = Some(ticked(text, number));
}

#[given(expr = "前回の台帳を渡していない")]
fn before_none(world: &mut InterviewWorld) {
    let handed_over = world.scene.previous.take();
    assert_eq!(handed_over, None, "no ledger is handed over");
}

#[given(expr = "前回の台帳として渡したファイルの課題の区切りが崩れていて、読み戻せない")]
async fn before_broken(world: &mut InterviewWorld) {
    issue_before(world, spot("I-0001"), ALL).await;
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
