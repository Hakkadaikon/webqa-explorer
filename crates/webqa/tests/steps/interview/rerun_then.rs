//! The Then steps of the re-run: what the ledger of the next run says of
//! the issues of the ledger before.

use cucumber::then;
use webqa_core::interview::ledger::{Compared, Entry, Goal};

use super::course::stashed;
use crate::ledger::read_back;
use crate::scene::found;
use crate::world::InterviewWorld;

/// The entry of the ledger the run wrote, for the number `example` stands for.
async fn entry_of(world: &mut InterviewWorld, example: &str) -> Entry {
    world.run().await;
    let number = world.scene.aliases.number(example);
    let ledger = found(world.ledger(), "the run wrote a ledger");
    let entries = ledger.entries.into_iter();
    found(
        entries.into_iter().find(|e| e.issue.number == number),
        "the issue is in the ledger",
    )
}

#[then(expr = "今回の台帳で {string} は残っているとして示されている")]
async fn is_remaining(world: &mut InterviewWorld, issue: String) {
    assert_eq!(
        entry_of(world, &issue).await.compared,
        Compared::Remaining,
        "compared"
    );
    drop(issue);
}

#[then(expr = "今回の台帳で {string} は新しく出たとして示されている")]
async fn is_new(world: &mut InterviewWorld, issue: String) {
    assert_eq!(
        entry_of(world, &issue).await.compared,
        Compared::New,
        "compared"
    );
    drop(issue);
}

#[then(expr = "今回の台帳で {string} は解消したとして示されている")]
async fn is_resolved(world: &mut InterviewWorld, issue: String) {
    assert_eq!(
        entry_of(world, &issue).await.compared,
        Compared::Resolved,
        "compared"
    );
    drop(issue);
}

#[then(expr = "今回の台帳で {string} は解消したとされず、確かめられなかったとして示されている")]
#[then(expr = "今回の台帳で {string} は確かめられなかったとして示されている")]
async fn is_unverified(world: &mut InterviewWorld, issue: String) {
    assert_eq!(
        entry_of(world, &issue).await.compared,
        Compared::Unverified,
        "compared"
    );
    drop(issue);
}

#[then(expr = "{string} は解消したとされていない")]
async fn is_not_resolved(world: &mut InterviewWorld, issue: String) {
    assert_ne!(
        entry_of(world, &issue).await.compared,
        Compared::Resolved,
        "compared"
    );
    drop(issue);
}

#[then(expr = "今回の台帳で {string} は選ばれている")]
async fn is_selected(world: &mut InterviewWorld, issue: String) {
    assert!(
        entry_of(world, &issue).await.selected,
        "the issue is selected"
    );
}

#[then(expr = "今回の台帳で {string} は選ばれていない")]
async fn is_not_selected(world: &mut InterviewWorld, issue: String) {
    assert!(
        !entry_of(world, &issue).await.selected,
        "the issue is not selected"
    );
}

#[then(expr = "今回の台帳の2件の課題は、どちらも新しく出たとして示されている")]
async fn both_new(world: &mut InterviewWorld) {
    world.run().await;
    let ledger = found(world.ledger(), "the run wrote a ledger");
    let compared: Vec<Compared> = ledger.entries.iter().map(|e| e.compared).collect();
    assert_eq!(compared, [Compared::New, Compared::New], "both are new");
}

#[then(expr = "試行は始まっていない")]
async fn no_trial_started(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(world.ran().launched, 0, "no browser opened");
    assert!(
        !world.kinds().contains(&"trial_started".to_owned()),
        "no trial started"
    );
}

#[then(expr = "前回の台帳と課題の番号で照らし合わせている")]
async fn compared_by_number(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(world.ran().config_error, None, "the ledgers were compared");
    let ledger = found(world.ledger(), "the run wrote a ledger");
    let compared: Vec<Compared> = ledger.entries.iter().map(|e| e.compared).collect();
    assert_eq!(compared, [Compared::Remaining], "the issue remains");
}

#[then(expr = "推測した文の違いは、照らし合わせの結果にも設定エラーにもなっていない")]
async fn guess_is_no_difference(world: &mut InterviewWorld) {
    world.run().await;
    let (before, now) = (stashed(world, "GUESS_BEFORE"), stashed(world, "GUESS_NOW"));
    assert_ne!(before, now, "the guesses differ");
    let goal = |guess: String| Goal::Inferred(vec![guess]);
    let text = found(
        world.before().store.ledger.as_deref(),
        "the run before wrote a ledger",
    );
    let heads = (
        read_back(text).head.goal,
        found(world.ledger(), "a ledger").head.goal,
    );
    assert_eq!(heads, (goal(before), goal(now)), "the guesses of the heads");
    assert_eq!(world.ran().config_error, None, "no configuration error");
}
