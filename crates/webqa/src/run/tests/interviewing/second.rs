//! Tests of the second call, which asks what the persona expects of the
//! operation the core carries out.

use webqa_core::interview::ask::preamble;

use super::unusable::complete;
use super::*;

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
    let asked = asked(judge, Some(interviewing(&[Op::Click], false)), &mut fakes).await;
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
        let asked = asked(judge, Some(interviewing(&[Op::Click], false)), &mut fakes).await;
        assert_eq!(asked.judge.asked.len(), calls, "{name}");
    }
}

#[tokio::test]
async fn missing_expectation_is_unreadable_after_two_calls() {
    let mut said = complete();
    said.table.remove(&QuestionId::new(EXPECTATION));
    let mut fakes = Fakes::new();
    let judge = Spy::giving(said);
    let asked = asked(judge, Some(interviewing(&[Op::Click], false)), &mut fakes).await;
    assert_eq!(
        (asked.judge.asked.len(), asked.reply),
        (2, Err(unreadable(EXPECTATION))),
        "the second call came back without it"
    );
}
