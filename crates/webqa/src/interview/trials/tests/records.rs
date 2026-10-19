//! Tests of what the loop writes, and of how it ends when the records
//! cannot be written.

use webqa_core::interview::compare::Mismatch;
use webqa_core::interview::ledger::Compared;
use webqa_core::interview::natural::{NATURAL, PERSONA_TRAIT, YES};
use webqa_core::ports::StoreError;
use webqa_core::redact::Rule;

use super::*;
use crate::run::RunError;

fn completed(confirmed: bool) -> Ending {
    Ending::Wrote {
        stopped: Stopped::Completed,
        confirmed,
    }
}

#[tokio::test]
async fn writes_only_ledger_events_report_screenshots() {
    let mut world = World::new(2);
    world.run().await;
    let store = &world.store;
    let counted = store.lines.len() + 1 + 1 + store.shots.len();
    assert_eq!(
        (store.writes, store.memories.len(), store.aside.len()),
        (counted, 0, 0),
        "the lines, one ledger, one report and the screenshots; no memory"
    );
}

#[tokio::test]
async fn confirmed_problem_is_found_in_the_trials() {
    let mut world = World::new(1);
    let mut page = top();
    page.console = vec!["TypeError: x is not a function".to_owned()];
    world.launch.browsers = VecDeque::from([FakeBrowser {
        pages: iter::repeat_n(page, 5).collect(),
        ..FakeBrowser::default()
    }]);
    let ending = world.run().await;
    assert_eq!(
        ending,
        completed(true),
        "a console error is a confirmed problem"
    );
}

#[tokio::test]
async fn records_that_cannot_be_written_fail_the_run() {
    // The first four writes are the lines of the run and its one trial;
    // then come the ledger, the last line and the report.
    for limit in [4, 5, 6] {
        let mut world = World::new(1);
        world.store.limit = Some(limit);
        assert_eq!(
            world.run().await,
            Ending::Failed(RunError::Store(StoreError)),
            "the write after {limit}"
        );
    }
}

#[tokio::test]
async fn second_interruption_while_writing_abandons() {
    // A first interruption ends the trials, so it is spent; a second one
    // abandons the records. Otherwise a second takes two.
    for (fire_on, spent) in [(vec![1, 2], true), (vec![3, 4], false)] {
        let mut world = World::new(1);
        world.store.hangs_on_ledger = true;
        let mut interrupt = Fires { calls: 0, fire_on };
        let ending = world.run_with(&mut interrupt).await;
        assert_eq!(
            (ending, world.store.ledger),
            (Ending::Abandoned, None),
            "the records were given up, after the first interruption: {spent}"
        );
    }
}

#[tokio::test]
async fn previous_ledger_of_another_persona_is_a_mismatch() {
    let mut world = World::new(1);
    let mut other = head(1);
    other.persona = "別の人".to_owned();
    world.plan.previous = Some(Ledger {
        head: other,
        entries: Vec::new(),
        unchecked_purposes: Vec::new(),
    });
    let ending = world.run().await;
    assert_eq!(
        (ending, world.store.ledger),
        (Ending::Mismatch(Mismatch::Persona), None),
        "nothing is written over a ledger of another persona"
    );
}

#[tokio::test]
async fn issues_of_the_previous_ledger_are_compared() {
    let mut before = World::new(1);
    before.judge.script = judged(hesitating(&["done"], 0.9));
    before.run().await;
    let mut now = World::new(1);
    now.plan.previous = before.ledger();
    now.run().await;
    let compared: Vec<_> = now
        .ledger()
        .unwrap()
        .entries
        .iter()
        .map(|e| e.compared)
        .collect();
    assert_eq!(
        compared,
        [Compared::Resolved],
        "reached again, and not hesitant now"
    );
}

/// The answers to the questions about an issue: natural, from the first trait.
fn judged(script: Scripted) -> Scripted {
    script
        .then(QuestionId::new(NATURAL), [(YES, 1.0)])
        .then(QuestionId::new(PERSONA_TRAIT), [("しないこと", 1.0)])
}

#[tokio::test]
async fn free_text_is_redacted_before_ledger_and_log() {
    let secret = "yamada-taro";
    let mut world = World::new(1);
    world.redactor = Redactor::new(&[Rule::literal("氏名", secret)]).unwrap();
    let said = format!("{secret}の画面が保存される");
    world.judge.script = judged(expecting(&["done"], 0.9, &said));
    world.run().await;
    let ledger = world.store.ledger.clone().unwrap();
    let log = world
        .store
        .lines
        .iter()
        .map(ToString::to_string)
        .collect::<String>();
    let report = world.store.report.clone().unwrap().to_string();
    assert_eq!(
        (
            ledger.contains(secret),
            log.contains(secret),
            report.contains(secret),
            ledger.contains("[[氏名]]")
        ),
        (false, false, false, true),
        "the name of the person is in no record, and the ledger says it was there"
    );
}
