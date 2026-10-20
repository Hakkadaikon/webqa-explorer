//! Steps of the trials that a limit or an interruption cuts, and what the
//! ledger counts of them.

use cucumber::{given, then, when};
use webqa_core::interview::issue::Kind;

use super::seen::TIME_UP;
use super::{End, course, ending_of, fill, first, found, issues_of, said, started_at};
use super::{minutes, step_name, think};
use crate::ledger::number_of;
use crate::world::InterviewWorld;

#[given(expr = "1回目の試行が行き詰まりで終わり、{string} の迷いの手がかりは1回目では起きていない")]
fn hesitation_not_in_first(world: &mut InterviewWorld, example: String) {
    let scene = &mut world.scene;
    first(scene, 0, End::GaveUp);
    let name = step_name(scene);
    let on = Some(("button", name.as_str(), 0));
    let number = number_of("/", &[("button", &name)], on, Kind::Hesitant);
    scene.aliases.bind(&example, number);
    drop(example);
}

#[given(expr = "2回目の試行で {string} の迷いの手がかりが起きたあと、利用者が実行を中断する")]
fn hesitation_then_interrupt(world: &mut InterviewWorld, example: String) {
    let scene = &mut world.scene;
    // The second operation fills the window; the wait for it never ends.
    scene.requests_per_hour = 4;
    scene.stall_sleep = true;
    let call = course(scene, 2, End::Cut);
    if let Some(reply) = scene.replies.get_mut(call) {
        *reply = Ok(said("click", 0.9));
    }
    drop(example);
}

#[then(expr = "台帳に {string} は出ていない")]
async fn issue_not_in_ledger(world: &mut InterviewWorld, example: String) {
    world.run().await;
    let ledger = found(world.ledger(), "the ledger was written");
    let number = world.scene.aliases.number(&example);
    let with = ledger
        .entries
        .iter()
        .filter(|entry| entry.issue.number == number);
    assert_eq!(with.count(), 0, "the issue of the cut trial");
    assert_eq!(
        issues_of(world, Kind::Hesitant),
        Vec::<usize>::new(),
        "no hesitation"
    );
}

#[then(expr = "台帳の試行の回数は {int} 回である")]
async fn ledger_trials(world: &mut InterviewWorld, n: usize) {
    world.run().await;
    let finished = found(world.ledger(), "the ledger was written")
        .head
        .finished;
    assert_eq!(finished, n, "the trials the ledger counts");
}

#[given(expr = "2回目の試行で、ブラウザを開いて最初のページを表示している最中である")]
fn second_is_opening(world: &mut InterviewWorld) {
    let scene = &mut world.scene;
    first(scene, 1, End::Goal);
    scene.hang_launch = Some(2);
    fill(scene);
}

#[then(expr = "最初のページの表示を待たずに、実行は時間切れとして終わっている")]
async fn ended_before_first_page(world: &mut InterviewWorld) {
    world.run().await;
    let ran = world.ran();
    assert_eq!(
        (ran.launched, ran.ending),
        (2, Some(TIME_UP)),
        "browsers, end"
    );
    assert_eq!(started_at(world, 2), None, "the second trial never began");
}

#[then(expr = "2回目の試行は打ち切りとして、試行の回数に数えられていない")]
async fn second_not_counted(world: &mut InterviewWorld) {
    world.run().await;
    let finished = found(world.ledger(), "the ledger was written")
        .head
        .finished;
    assert_eq!(finished, 1, "only the first trial counts");
}

#[given(expr = "1回目の試行が行き詰まりで終わり、2回目の試行の途中である")]
fn second_in_progress(world: &mut InterviewWorld) {
    let scene = &mut world.scene;
    // The second operation fills the window of the second trial.
    scene.requests_per_hour = 4;
    first(scene, 0, End::GaveUp);
    course(scene, 2, End::Cut);
}

#[when(expr = "2回目の試行が 実行全体の期限による時間切れ で終わる")]
async fn second_times_out_by_limit(world: &mut InterviewWorld) {
    let scene = &mut world.scene;
    scene.max_duration = Some(minutes(30));
    // The call after the first trial's, which was the first trial's own.
    think(scene, 1, minutes(31));
    world.run().await;
    ended_as(world, "time_up");
}

#[when(expr = "2回目の試行が 利用者による中断 で終わる")]
async fn second_is_interrupted(world: &mut InterviewWorld) {
    world.scene.stall_sleep = true;
    world.run().await;
    ended_as(world, "interrupted");
}

/// The second trial ended as `ending`, as the log names it.
fn ended_as(world: &InterviewWorld, ending: &str) {
    assert_eq!(ending_of(world, 2).as_deref(), Some(ending), "how it ended");
}

#[then(expr = "目的に届かない試行は {int} 回だけ数えられている")]
async fn unreached_counted(world: &mut InterviewWorld, n: usize) {
    world.run().await;
    let counts = issues_of(world, Kind::Unreached);
    assert_eq!(counts, [n], "the unreached issue's count");
}

#[given(expr = "1回目の試行の途中で、利用者が実行を中断する")]
fn first_interrupted(world: &mut InterviewWorld) {
    first(&mut world.scene, 1, End::Open);
}

#[then(expr = "台帳に課題は出ておらず、出現の割合も書かれていない")]
async fn no_issue_no_share(world: &mut InterviewWorld) {
    world.run().await;
    let ledger = found(world.ledger(), "the ledger was written");
    assert_eq!(ledger.entries, [], "no issue");
    let text = found(world.ran().store.ledger.clone(), "the ledger text");
    let shares = text.lines().filter(|line| line.contains("起きた回数"));
    assert_eq!(shares.count(), 0, "no line of how often");
}

#[then(expr = "台帳に、終えた試行が無いことが書かれている")]
async fn no_trial_written(world: &mut InterviewWorld) {
    world.run().await;
    let text = found(world.ran().store.ledger.clone(), "the ledger text");
    let line = "- 終えた試行: 0(終えた試行が無いので、課題と出現の割合を出していない)";
    let said = text.lines().filter(|written| *written == line);
    assert_eq!(said.count(), 1, "the head says so");
}

#[given(expr = "1回目の試行が終わり、2回目の試行の途中である")]
fn second_midway(world: &mut InterviewWorld) {
    let scene = &mut world.scene;
    first(scene, 1, End::Goal);
    course(scene, 1, End::Open);
}

#[then(expr = "中断した2回目の試行は、目的に届かない試行に数えられていない")]
async fn interrupted_not_unreached(world: &mut InterviewWorld) {
    world.run().await;
    let ended = ending_of(world, 2);
    assert_eq!(ended.as_deref(), Some("interrupted"), "how the trial ended");
    let counts = issues_of(world, Kind::Unreached);
    assert_eq!(counts, Vec::<usize>::new(), "no unreached issue");
}
