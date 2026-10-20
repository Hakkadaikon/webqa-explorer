//! Tests of the second call at a step where the core passes over the first
//! element ranked, and of the calls the run keeps.

use std::iter;

use super::second::expecting_click;
use super::unusable::complete;
use super::*;
use crate::run::tests::{Never, page};

/// What the judge answers at a step that clicks, ranking the elements `1`
/// then `2`, or at one that is done.
fn step(script: Scripted, op: &str) -> Scripted {
    let script = script
        .then(QuestionId::operation(), [(op, 0.9)])
        .then(QuestionId::target(), [("1", 0.8), ("2", 0.6)])
        .then(QuestionId::error_shown(), [("true", 0.0)])
        .then(QuestionId::goal_reached(), [("true", 0.0)])
        .then(QuestionId::new(HESITATION), [("true", 0.2)])
        .then_text(QuestionId::new(PURPOSE_GUESS), "経費を申請する");
    if op == "done" {
        script
    } else {
        script.then_text(QuestionId::new(EXPECTATION), "保存される")
    }
}

/// What a run of two clicks and a done on a page that never changes did.
struct Ran {
    /// What the judge was asked in each call.
    asked: Vec<Vec<Question>>,
    /// The step of each call the run kept.
    steps: Vec<u64>,
    /// The lines of the log that say the judge was asked.
    lines: usize,
}

/// The judge clicks twice, then is done.
fn clicks_then_done() -> Scripted {
    let steps = ["click", "click", "done"];
    steps.into_iter().fold(Scripted::default(), step)
}

/// The fake ports with `judge` in place of the fake judge.
fn ported<'a>(fakes: &'a mut Fakes, judge: &'a mut Spy) -> Ports<'a> {
    Ports::new(
        &mut fakes.browser,
        judge,
        &mut fakes.clock,
        &mut fakes.store,
        &fakes.redactor,
    )
}

async fn ran() -> Ran {
    let mut fakes = Fakes::new();
    fakes.browser.pages = iter::repeat_n(page("/"), 40).collect();
    let mut judge = Spy::new(false, clicks_then_done());
    let interview = interviewing(&[Op::Click], false);
    let mut ports = ported(&mut fakes, &mut judge).interviewing(interview);
    let rules = scenario(&[Op::Click]);
    let mut never = Never;
    let run = explore(&mut ports, &mut never, rules, Memory::default());
    run.await.unwrap();
    let steps = ports.record().calls.iter().map(|c| c.step).collect();
    let lines = fakes.kinds().iter().filter(|k| *k == "asked").count();
    Ran {
        asked: judge.asked,
        steps,
        lines,
    }
}

#[tokio::test]
async fn second_call_names_the_element_the_core_picked() {
    let ran = ran().await;
    let named: Vec<_> = ran.asked.into_iter().filter(|q| q.len() == 1).collect();
    let second = "2番「[[メールアドレス]] に送る」";
    assert_eq!(
        named,
        [[expecting_click("1番「保存」")], [expecting_click(second)]],
        "the same page twice: the second click passes over the element left unmoved"
    );
}

#[tokio::test]
async fn each_call_is_logged_and_recorded_in_order() {
    let ran = ran().await;
    assert_eq!(
        (ran.steps, ran.lines),
        (vec![1, 1, 2, 2, 3], 5),
        "two calls for each click and one for the done, all of the step's number"
    );
}

/// A judge that clicks, and whose calls with one question also say the
/// operation is `scroll_down`.
fn straying() -> Spy {
    let mut again = complete();
    let scroll = vec![("scroll_down".to_owned(), 0.9)];
    again.table.insert(QuestionId::operation(), scroll);
    again.table.remove(&QuestionId::target());
    let mut judge = Spy::giving(complete());
    judge.later = Some(again);
    judge
}

#[tokio::test]
async fn second_call_cannot_change_the_operation() {
    let mut fakes = Fakes::new();
    fakes.browser.pages = iter::repeat_n(page("/"), 40).collect();
    let mut judge = straying();
    let interview = interviewing(&[Op::Click], false);
    let mut ports = ported(&mut fakes, &mut judge).interviewing(interview);
    let rules = scenario(&[Op::Click, Op::ScrollDown]);
    let mut never = Never;
    let run = explore(&mut ports, &mut never, rules, Memory::default());
    run.await.unwrap();
    // The core reloads by itself when the page stays; the judge's are clicks.
    let ops = fakes.browser.touches.iter().map(|touch| touch.op.as_str());
    let mut judged: Vec<&str> = ops.filter(|op| *op != "reload").collect();
    judged.dedup();
    assert_eq!(judged, ["click"], "the operation of the first call");
}
