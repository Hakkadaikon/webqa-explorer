//! Tests of the second call, which asks what the persona expects of the
//! operation the core carries out.

use webqa_core::interview::ask::preamble;

use std::iter;

use super::unmoved::ported;
use super::unusable::complete;
use super::*;
use crate::run::tests::{Never, page};

/// The question the second call asks about clicking element `target`.
pub(super) fn expecting_click(target: &str) -> Question {
    let lead = preamble(&persona(), "経費を申請する");
    let ask = format!(
        "あなたは次に『click』をする。対象: {target}。この操作で何が起きると期待するか、1文で答える。"
    );
    Question::Text {
        id: QuestionId::new(EXPECTATION),
        prompt: format!("{lead}\n\n{ask}"),
    }
}

fn unreadable(id: &str) -> JudgeError {
    JudgeError::Unreadable {
        question: id.to_owned(),
    }
}

#[tokio::test]
async fn second_call_asks_only_the_expectation_of_the_chosen_operation() {
    let mut fakes = Fakes::new();
    let judge = Spy::new(false, answers(false));
    let asked = asked(judge, Some(interviewing(false)), &mut fakes).await;
    let ids = |questions: &Vec<Question>| -> Vec<String> {
        questions
            .iter()
            .map(|q| q.id().as_str().to_owned())
            .collect()
    };
    let calls: Vec<_> = asked.judge.asked.iter().map(ids).collect();
    assert_eq!(calls.len(), 2, "two calls");
    assert!(
        !calls[0].contains(&EXPECTATION.to_owned()),
        "not in the first"
    );
    assert_eq!(
        asked.judge.asked[1],
        [expecting_click("1番「保存」")],
        "the operation and the element, as the screen lists it"
    );
}

#[tokio::test]
async fn second_call_is_not_made_when_no_operation_is_carried_out() {
    let say = |op: &str| {
        let mut said = complete();
        said.table
            .insert(QuestionId::operation(), vec![(op.to_owned(), 0.9)]);
        said
    };
    let mut no_hesitation = complete();
    no_hesitation.table.remove(&QuestionId::new(HESITATION));
    let cases = [
        ("click", complete(), 2),
        ("done", say("done"), 1),
        (
            "an operation the settings do not offer",
            say("scroll_down"),
            1,
        ),
        (
            "an added answer of the first call missing",
            no_hesitation,
            1,
        ),
    ];
    for (name, answers, calls) in cases {
        let mut fakes = Fakes::new();
        let judge = Spy::giving(answers);
        let asked = asked(judge, Some(interviewing(false)), &mut fakes).await;
        assert_eq!(asked.judge.asked.len(), calls, "{name}");
    }
}

#[tokio::test]
async fn missing_expectation_is_unreadable_after_two_calls() {
    let mut said = complete();
    said.table.remove(&QuestionId::new(EXPECTATION));
    let mut fakes = Fakes::new();
    let judge = Spy::giving(said);
    let asked = asked(judge, Some(interviewing(false)), &mut fakes).await;
    assert_eq!(
        (asked.judge.asked.len(), asked.reply),
        (2, Err(unreadable(EXPECTATION))),
        "the second call came back without it"
    );
}

/// A judge that answers the first call and fails the second.
fn failing_second(failure: JudgeError) -> Spy {
    let mut judge = Spy::giving(complete());
    judge.later = Some(Err(failure));
    judge
}

#[tokio::test]
async fn failed_second_call_reaches_the_core_as_it_failed() {
    let cases = [JudgeError::Timeout, JudgeError::Api, JudgeError::Bridge];
    for failure in cases {
        let mut fakes = Fakes::new();
        let judge = failing_second(failure.clone());
        let asked = asked(judge, Some(interviewing(false)), &mut fakes).await;
        let kept: Vec<_> = asked
            .steps
            .iter()
            .map(|taken| taken.reply.is_none())
            .collect();
        assert_eq!(
            (asked.judge.asked.len(), asked.reply, kept),
            (2, Err(failure.clone()), vec![true]),
            "{failure:?}: not read as an unusable answer, and no reply kept"
        );
    }
}

/// The step of each call the run kept, and the lines of the log that say the
/// judge was asked, of a run whose second call always fails.
async fn run_with_failing_second() -> (Vec<u64>, usize) {
    let mut fakes = Fakes::new();
    fakes.browser.pages = iter::repeat_n(page("/"), 40).collect();
    let mut judge = failing_second(JudgeError::Timeout);
    let interview = interviewing(false);
    let mut ports = ported(&mut fakes, &mut judge).interviewing(interview);
    let mut never = Never;
    let rules = scenario(&[Op::Click]);
    let _ = explore(&mut ports, &mut never, rules, Memory::default()).await;
    let steps = ports.record().calls.iter().map(|c| c.step).collect();
    (
        steps,
        fakes.kinds().iter().filter(|k| *k == "asked").count(),
    )
}

#[tokio::test]
async fn both_calls_are_kept_when_the_second_failed() {
    assert_eq!(
        run_with_failing_second().await,
        (vec![1, 1, 2, 2, 3, 3, 4, 4, 5, 5], 10),
        "two calls for each step, though the second always failed"
    );
}
