//! The Then steps of the purposes of screens about what the judge was
//! asked.

use cucumber::then;
use serde_json::Value;
use webqa_core::interview::ask::PURPOSE_MATCH;
use webqa_core::interview::issue::Kind;
use webqa_core::interview::ledger::{Entry, Ledger};
use webqa_core::interview::natural::{NATURAL, PERSONA_TRAIT};
use webqa_core::judge::Question;

use crate::judge::Asked;
use crate::scene::found;
use crate::world::InterviewWorld;

/// The calls that carry the question `id`, in order.
pub(super) fn asks<'a>(world: &'a InterviewWorld, id: &str) -> Vec<&'a Asked> {
    let carries = |asked: &&Asked| asked.questions.iter().any(|q| q.id().as_str() == id);
    world.ran().judge.asked.iter().filter(carries).collect()
}

/// The prompt of a question, as the judge reads it.
pub(super) fn prompt(question: &Question) -> String {
    let value = serde_json::to_value(question).unwrap_or_default();
    let prompt = value.get("prompt").and_then(Value::as_str);
    found(prompt, "a question has a prompt").to_owned()
}

/// The lines of the one mismatch question, after `lead`, which they begin
/// with.
pub(super) fn question_lines(world: &InterviewWorld, lead: &str) -> Vec<String> {
    let calls = asks(world, PURPOSE_MATCH);
    let call = found(calls.first(), "a mismatch question was asked");
    let question = found(call.questions.first(), "the call has a question");
    let text = prompt(question);
    let lines = text.lines().filter_map(|line| line.strip_prefix(lead));
    lines.map(str::to_owned).collect()
}

/// The ledger the run wrote.
pub(super) fn ledger_of(world: &InterviewWorld) -> Ledger {
    found(world.ledger(), "the run wrote a ledger")
}

/// The issues of the ledger that are about a mismatch.
pub(super) fn mismatches(ledger: &Ledger) -> Vec<&Entry> {
    let all = ledger.entries.iter();
    all.filter(|entry| entry.issue.kind() == Kind::Mismatch)
        .collect()
}

/// The one mismatch issue of the ledger.
pub(super) fn the_mismatch(ledger: &Ledger) -> &Entry {
    let all = mismatches(ledger);
    assert_eq!(all.len(), 1, "one mismatch issue");
    found(all.first(), "a mismatch issue")
}

#[then(expr = "ずれの問いには、推測 {string} と {string} と、書いた目的 {string} が渡されている")]
async fn question_carries(world: &mut InterviewWorld, one: String, other: String, purpose: String) {
    world.run().await;
    let guesses = question_lines(world, "- ");
    assert_eq!(guesses, [one, other], "the guesses passed");
    let written = question_lines(world, "書いた目的: ");
    assert_eq!(written, [purpose], "the purpose written");
}

/// The mismatch question was asked once.
async fn mismatch_asked_once(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(asks(world, PURPOSE_MATCH).len(), 1, "asked once");
}

#[then(expr = "ずれの問いは、その URL の型について1回だけ聞かれている")]
async fn asked_once(world: &mut InterviewWorld) {
    mismatch_asked_once(world).await;
}

#[then(expr = "URL の型 {string} のずれの問いは1回だけ聞かれている")]
async fn asked_once_for(world: &mut InterviewWorld, pattern: String) {
    mismatch_asked_once(world).await;
    let text = prompt(first_question(world, PURPOSE_MATCH));
    let head = found(text.lines().next(), "the question has a first line");
    let about = format!("URL の型 {pattern} の画面には、次の目的を書いた。");
    assert_eq!(head, about, "the question is about the pattern");
}

#[then(expr = "その問いに渡した推測は {string} と {string} の2つである")]
async fn guesses_passed(world: &mut InterviewWorld, one: String, other: String) {
    world.run().await;
    assert_eq!(question_lines(world, "- "), [one, other], "the guesses");
}

#[then(expr = "台帳に、目的のずれの課題は出ていない")]
async fn ledger_has_no_mismatch(world: &mut InterviewWorld) {
    world.run().await;
    assert!(
        mismatches(&ledger_of(world)).is_empty(),
        "no mismatch issue"
    );
}

#[then(expr = "判断モデルに、その URL の型のずれの問いは聞かれていない")]
async fn not_asked(world: &mut InterviewWorld) {
    world.run().await;
    assert!(
        asks(world, PURPOSE_MATCH).is_empty(),
        "no mismatch question"
    );
    assert!(!world.ran().judge.asked.is_empty(), "the judge was asked");
}

#[then(expr = "ずれの問いは {int} 回だけ聞かれ、自然さの問いは聞かれていない")]
async fn asked_only(world: &mut InterviewWorld, times: usize) {
    world.run().await;
    assert_eq!(
        asks(world, PURPOSE_MATCH).len(),
        times,
        "the mismatch questions"
    );
    assert!(asks(world, NATURAL).is_empty(), "no naturalness question");
}

/// The first question of the first call that carries the question `id`.
fn first_question<'a>(world: &'a InterviewWorld, id: &str) -> &'a Question {
    let calls = asks(world, id);
    let call = found(calls.first(), "the question was asked");
    found(call.questions.first(), "the call has a question")
}

#[then(
    expr = "ずれの問いのあとに、その目的のずれの課題についてペルソナの性質の問いだけが1回聞かれている"
)]
async fn trait_after_mismatch(world: &mut InterviewWorld) {
    world.run().await;
    let all = &world.ran().judge.asked;
    let at = |id: &str| {
        all.iter()
            .position(|a| a.questions.iter().any(|q| q.id().as_str() == id))
    };
    let mismatch = found(at(PURPOSE_MATCH), "the mismatch question was asked");
    let asked = found(at(PERSONA_TRAIT), "the trait question was asked");
    assert_eq!(asked, mismatch + 1, "right after");
    assert_eq!(asks(world, PERSONA_TRAIT).len(), 1, "asked once");
    assert!(asks(world, NATURAL).is_empty(), "no naturalness question");
    assert_eq!(
        about_mismatch(world, PERSONA_TRAIT),
        1,
        "about the mismatch"
    );
}

/// How many lines of the first question `id` name the written pattern as
/// the screen of the issue.
fn about_mismatch(world: &InterviewWorld, id: &str) -> usize {
    let written = world.scene.purpose.screens.first();
    let pattern = found(written, "a purpose is written").pattern.as_str();
    let place = format!("画面: URL の型 {pattern}");
    let text = prompt(first_question(world, id));
    text.lines().filter(|line| *line == place).count()
}
