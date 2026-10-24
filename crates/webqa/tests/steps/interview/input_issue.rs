//! The Then steps of the purposes of screens about the issue in the ledger.

use cucumber::then;
use serde_json::Value;
use webqa::store::screenshots::issue_name;
use webqa_core::interview::issue::{
    Count, Detail, Key, Kind, Number, Place, Shot, Spot, UncheckedPurpose,
};
use webqa_core::interview::natural::{PersonaTrait, Section, Trait};

use super::steps_scripted;
use super::thens::{ledger_of, question_lines, the_mismatch};
use crate::scene::found;
use crate::world::InterviewWorld;

#[then(expr = "台帳に、URL の型 {string} の、種類が目的のずれの課題が出ている")]
async fn ledger_has_mismatch(world: &mut InterviewWorld, pattern: String) {
    world.run().await;
    let ledger = ledger_of(world);
    let issue = &the_mismatch(&ledger).issue;
    assert_eq!(issue.place, Place::Pattern(pattern.clone()), "its place");
    let key = Key {
        spot: Spot::Pattern(&pattern),
        kind: Kind::Mismatch,
    };
    assert_eq!(issue.number, Number::of(key), "its number");
}

#[then(expr = "その課題には、書いた目的と、渡した推測が書かれている")]
async fn issue_has_purpose_and_guesses(world: &mut InterviewWorld) {
    world.run().await;
    let guesses = question_lines(world, "- ");
    let written = world.scene.purpose.screens.first();
    let purpose = found(written, "a purpose is written").purpose.clone();
    let ledger = ledger_of(world);
    let detail = &the_mismatch(&ledger).issue.detail;
    assert_eq!(detail, &Detail::Mismatch { purpose, guesses }, "the detail");
}

#[then(expr = "目的のずれの課題の画面は URL の型 {string} で、対象の要素は無い")]
async fn issue_is_at_the_pattern(world: &mut InterviewWorld, pattern: String) {
    world.run().await;
    let ledger = ledger_of(world);
    let issue = &the_mismatch(&ledger).issue;
    assert_eq!(issue.place, Place::Pattern(pattern), "its place");
    assert_eq!(issue.target, None, "no element");
}

#[then(
    expr = "その課題は、2回の試行のうち2回起きたと書かれ、その画面に届いた試行の数は {int} である"
)]
async fn issue_counts(world: &mut InterviewWorld, reached: usize) {
    world.run().await;
    let ledger = ledger_of(world);
    let count = the_mismatch(&ledger).issue.count;
    let expected = Count {
        occurred: reached,
        finished: reached,
        reached,
    };
    assert_eq!(count, expected, "the counts");
    assert_eq!(ledger.head.finished, 3, "every trial finished");
}

#[then(
    expr = "その課題のスクリーンショットは1回目の試行のステップ {int} のもので、名前は課題の番号、{int}、{int} から作られている"
)]
async fn issue_shot(world: &mut InterviewWorld, first: u64, trial: usize, again: u64) {
    world.run().await;
    let ledger = ledger_of(world);
    let issue = &the_mismatch(&ledger).issue;
    assert_eq!(trial, 1, "the name is made of the first trial");
    let shot = Shot { trial, step: first };
    assert_eq!(again, first, "the name is made of the step of the shot");
    assert_eq!(issue.shot, Some(shot), "the issue points at the step");
    let written: Vec<&str> = world
        .ran()
        .store
        .shots
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(written, [issue_name(issue.number, shot)], "the file");
}

/// The screen purposes of the ledger that were not checked.
fn unchecked(world: &InterviewWorld) -> Vec<UncheckedPurpose> {
    ledger_of(world).unchecked_purposes
}

#[then(expr = "URL の型 {string} の項目は、確かめられなかった区分にある")]
async fn unchecked_screen(world: &mut InterviewWorld, pattern: String) {
    world.run().await;
    let written = world.scene.purpose.screens.first();
    let purpose = found(written, "a purpose is written").purpose.clone();
    let expected = UncheckedPurpose { pattern, purpose };
    assert_eq!(unchecked(world), [expected], "the unchecked purposes");
}

#[then(expr = "3つ目の項目は、確かめられなかった区分にある")]
async fn third_is_unchecked(world: &mut InterviewWorld) {
    world.run().await;
    let screens = &world.scene.purpose.screens;
    let third = found(screens.get(2), "three purposes are written");
    let expected = UncheckedPurpose {
        pattern: third.pattern.as_str().to_owned(),
        purpose: third.purpose.clone(),
    };
    assert_eq!(unchecked(world), [expected], "the unchecked purposes");
}

/// Whether `call` of the report is the second of a step: the answers hold
/// what is expected and no operation.
fn is_second_call(call: &Value) -> bool {
    let answers = call.get("answers");
    answers.is_some_and(|a| a.is_object() && a.get("operation").is_none())
}

#[then(expr = "試行の中の判断予算の使った回数は、ずれの問いで増えていない")]
async fn budget_is_not_spent(world: &mut InterviewWorld) {
    world.run().await;
    let report = found(world.ran().store.report.as_ref(), "the report was written");
    let calls = found(
        report.get("calls").and_then(Value::as_array),
        "it lists calls",
    );
    let step_of = |call: &Value| call.get("step").and_then(Value::as_u64).unwrap_or(0);
    let in_trials = calls
        .iter()
        .filter(|call| step_of(call) != 0 && !is_second_call(call))
        .count();
    assert_eq!(
        in_trials,
        steps_scripted(&world.scene),
        "the steps of the trials"
    );
}

#[then(expr = "判断モデルの台本が自然でないと答える内容でも、その課題は台帳の本体に入る")]
async fn mismatch_in_body(world: &mut InterviewWorld) {
    world.run().await;
    let ledger = ledger_of(world);
    let entry = the_mismatch(&ledger);
    assert_eq!(entry.section, Section::Body, "its section");
    let persona = &world.scene.persona;
    let content = persona.it_literacy.clone();
    let expected = PersonaTrait {
        which: Trait::ItLiteracy,
        content,
    };
    assert_eq!(entry.persona_trait, Some(expected), "its trait");
}
