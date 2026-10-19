//! Tests of the issues the ledger gets from the trials and from the
//! questions after them.

use webqa_core::interview::issue::{Count, Kind};
use webqa_core::interview::ledger::Entry;
use webqa_core::interview::natural::{
    NATURAL, NO, PERSONA_TRAIT, PersonaTrait, Section, Trait, YES,
};

use super::windows::slow_run;
use super::*;

/// A world of one trial in which the judge says it is done at once,
/// hesitating with the probability `hesitation`.
fn hesitant(hesitation: f64) -> World {
    let mut world = World::new(1);
    world.judge.script = hesitating(&["done"], hesitation);
    world
}

/// The judge's answers to the questions about an issue: whether it is
/// natural and which trait of the persona it comes from.
fn judged(script: Scripted, natural: &str, from: &str) -> Scripted {
    script
        .then(QuestionId::new(NATURAL), [(natural, 1.0)])
        .then(QuestionId::new(PERSONA_TRAIT), [(from, 1.0)])
}

/// The kind, section and count of each entry of the ledger written.
pub(super) fn entries(world: &World) -> Vec<(Kind, Section, Count)> {
    let ledger = world.ledger().unwrap();
    let described = |entry: &Entry| (entry.issue.kind(), entry.section, entry.issue.count);
    ledger.entries.iter().map(described).collect()
}

pub(super) const ONCE: Count = Count {
    occurred: 1,
    finished: 1,
    reached: 1,
};

#[tokio::test]
async fn hesitation_is_asked_about_and_placed() {
    let mut world = hesitant(0.9);
    world.judge.script = judged(hesitating(&["done"], 0.9), NO, "ITの慣れ");
    world.run().await;
    let ledger = world.ledger().unwrap();
    let traits: Vec<_> = ledger
        .entries
        .iter()
        .map(|e| e.persona_trait.clone())
        .collect();
    let it_literacy = PersonaTrait {
        which: Trait::ItLiteracy,
        content: "苦手".to_owned(),
    };
    assert_eq!(
        (entries(&world), traits),
        (
            vec![(Kind::Hesitant, Section::OffPersona, ONCE)],
            vec![Some(it_literacy)]
        ),
        "the issue is off the persona, from the trait the judge named"
    );
}

#[tokio::test]
async fn natural_hesitation_stays_in_the_body() {
    let mut world = hesitant(0.9);
    world.judge.script = judged(hesitating(&["done"], 0.9), YES, "しないこと");
    world.run().await;
    assert_eq!(
        entries(&world),
        [(Kind::Hesitant, Section::Body, ONCE)],
        "a natural hesitation is an issue of the body"
    );
}

#[tokio::test]
async fn unusable_answer_leaves_the_issue_unchecked() {
    let mut world = hesitant(0.9);
    world.run().await;
    let ledger = world.ledger().unwrap();
    assert_eq!(
        (entries(&world), ledger.entries[0].persona_trait.clone()),
        (vec![(Kind::Hesitant, Section::Unchecked, ONCE)], None),
        "no answer, no trait"
    );
}

#[tokio::test]
async fn no_hesitation_no_issue() {
    let mut world = hesitant(0.0);
    world.run().await;
    assert_eq!(
        entries(&world),
        Vec::new(),
        "nothing hesitant, nothing asked"
    );
}

#[tokio::test]
async fn cut_trial_is_not_counted_in_the_issues() {
    let mut world = World::new(3);
    let ops = [vec!["click", "done"], vec!["click", "click"]].concat();
    let ending = slow_run(&mut world, hesitating(&ops, 0.9)).await;
    let timed_out = Ending::Wrote {
        stopped: Stopped::TimeUp,
        confirmed: false,
    };
    let kinds: Vec<_> = entries(&world)
        .iter()
        .map(|(kind, _, count)| (*kind, *count))
        .collect();
    let once = Count {
        occurred: 1,
        finished: 1,
        reached: 1,
    };
    assert_eq!(
        (
            ending,
            kinds.len(),
            kinds.iter().all(|(_, count)| *count == once)
        ),
        (timed_out, 2, true),
        "the hesitation of the cut trial is in no count and no issue"
    );
}

#[tokio::test]
async fn trial_that_ran_out_of_answers_is_an_unreached_issue() {
    let mut world = World::new(1);
    world.plan.rules.max_decisions = nz(3);
    world.judge.script = script(&["click"]);
    world.run().await;
    assert_eq!(
        entries(&world),
        [(Kind::Unreached, Section::Unchecked, ONCE)],
        "the steps without usable answers carry no hesitation, and the trial did not get there"
    );
}
