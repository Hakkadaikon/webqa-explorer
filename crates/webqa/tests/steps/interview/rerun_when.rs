//! The When steps of the re-run: the scene the next run is set to.

use cucumber::when;
use webqa_core::interview::issue::Kind;

use super::course::{GUESS, Mark, Spot, at, course, number, screen, spot, stash, trials_at};
use super::given::{bound, issue_before};
use crate::world::InterviewWorld;

/// Sets the scene to run `trials_at(total, reached, 0, at)` with the ledger
/// before: the issue does not happen.
fn rerun(world: &mut InterviewWorld, total: u32, reached: u32, occurred: u32, on: Spot) {
    course(
        &mut world.scene,
        &trials_at(total, reached, occurred, on),
        GUESS,
    );
}

#[when(expr = "同じペルソナと目的で前回の台帳を渡して再実行し、{string} がまた出る")]
#[when(expr = "前回の台帳を渡して再実行し、{string} がまた出る")]
fn again_has(world: &mut InterviewWorld, issue: String) {
    rerun(world, 3, 3, 3, spot(issue));
}

#[when(expr = "同じペルソナと目的で前回の台帳を渡して再実行し、{string} が出る")]
#[when(expr = "前回の台帳を渡して再実行し、新しい課題 {string} が出る")]
async fn again_new(world: &mut InterviewWorld, issue: String) {
    if world.scene.previous.is_none() {
        issue_before(world, spot("I-0001"), 3, 3).await;
    }
    let at = bound(world, issue);
    rerun(world, 3, 3, 3, at);
}

#[when(
    expr = "前回の台帳を渡して {int} 回試し、そのうち {int} 回が {string} に届き、{string} は1回も起きない"
)]
fn again_reaching(
    world: &mut InterviewWorld,
    total: u32,
    reached: u32,
    screen: String,
    issue: String,
) {
    rerun(world, total, reached, 0, at(issue, screen));
}

#[when(
    expr = "前回の台帳を渡して3回試し、そのうち1回だけが {string} に届き、{string} は1回も起きない"
)]
fn again_once(world: &mut InterviewWorld, screen: String, issue: String) {
    rerun(world, 3, 1, 0, at(issue, screen));
}

#[when(expr = "前回の台帳を渡して再実行し、どの試行も {string} に届かない")]
fn again_missing(world: &mut InterviewWorld, name: String) {
    rerun(world, 3, 0, 0, screen(name));
}

#[when(expr = "インタビューのモードで実行し、課題が2件出る")]
fn two_issues(world: &mut InterviewWorld) {
    let walk = vec![
        (spot("I-0001"), Mark::Hesitant),
        (spot("I-0002"), Mark::Hesitant),
    ];
    for issue in ["I-0001", "I-0002"] {
        world
            .scene
            .aliases
            .bind(issue, number(spot(issue), Kind::Hesitant));
    }
    course(&mut world.scene, &[walk.clone(), walk.clone(), walk], GUESS);
}

#[when(expr = "前回の台帳を渡して再実行しようとする")]
fn again_trying(world: &mut InterviewWorld) {
    rerun(world, 3, 3, 3, spot("I-0001"));
}

#[when(expr = "同じ開始の URL で目的を推測させて再実行し、推測した文が {string} になる")]
fn again_guessing(world: &mut InterviewWorld, guess: String) {
    course(
        &mut world.scene,
        &trials_at(3, 3, 3, spot("I-0001")),
        &guess,
    );
    stash(world, "GUESS_NOW", &guess);
    drop(guess);
}
