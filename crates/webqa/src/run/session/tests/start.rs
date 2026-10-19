//! Tests of where the memory of a run comes from.

use webqa_core::explore::ElementOp;
use webqa_core::judge::{Answers, JudgeError, Question};
use webqa_core::ports::Pending;
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;

use super::*;

/// Records the options of the questions it is asked, then answers from the script.
struct Questioned {
    asked: Vec<Question>,
    script: Scripted,
}

impl Judge for Questioned {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        self.asked.extend_from_slice(questions);
        self.script.ask(screen, questions)
    }
}

/// The text of a memory in which `保存` is operated on the top page.
fn saved_text() -> String {
    let mut memory = Memory::default();
    let screen = Redactor::default().screen(top().screen);
    let at = memory.visit(&screen, &BTreeSet::new());
    memory.record(at, &save(), ElementOp::Click, at);
    memory.to_json(ORIGIN).to_string()
}

/// The options of the first question about the target, after the run of
/// `start` on a store that holds [`saved_text`].
async fn first_targets(start: Start, store: &mut MemoryStore) -> Vec<String> {
    let (mut browser, mut clock) = (browser_on(&top()), FakeClock::default());
    let mut judge = Questioned {
        asked: Vec::new(),
        script: clicks(),
    };
    let redactor = Redactor::default();
    let mut ports = Ports::new(&mut browser, &mut judge, &mut clock, store, &redactor);
    let plan = Plan { start, ..plan() };
    let mut user = Counting {
        calls: 0,
        fire_on: BTreeSet::from([10]),
    };
    conduct(&mut ports, &mut user, rules(100), &plan).await;
    judge
        .asked
        .iter()
        .find_map(target_options)
        .cloned()
        .unwrap()
}

/// The options of a question about the target.
fn target_options(question: &Question) -> Option<&Vec<String>> {
    match question {
        Question::Choice { id, options, .. } => (*id == QuestionId::target()).then_some(options),
        Question::Noul { .. } | Question::Score { .. } | Question::Text { .. } => None,
    }
}

fn stored() -> MemoryStore {
    let mut store = MemoryStore::default();
    store.memories.insert(ORIGIN.to_owned(), saved_text());
    store
}

#[tokio::test]
async fn replay_starts_from_given_memory_and_keeps_stored() {
    let mut store = stored();
    let targets = first_targets(Start::Given(Memory::default()), &mut store).await;
    assert_eq!(targets, ["1", "2", "3", "4"], "保存 is still an option");
    assert_eq!(
        store.memories[ORIGIN],
        saved_text(),
        "the stored one is left alone"
    );
}

#[tokio::test]
async fn stored_start_is_written_back() {
    let mut store = stored();
    let targets = first_targets(Start::Stored, &mut store).await;
    assert_eq!(targets, ["2", "3", "4"], "保存 was operated before");
    assert_ne!(
        store.memories[ORIGIN],
        saved_text(),
        "written back at the end"
    );
    assert!(saved_operated(&store));
}
