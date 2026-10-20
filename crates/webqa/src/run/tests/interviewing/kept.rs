//! Tests of the steps the interview keeps.

use super::*;

#[tokio::test]
async fn interview_keeps_the_reply_of_the_step() {
    let mut fakes = Fakes::new();
    let judge = Spy::new(false, answers(false));
    let asked = asked(judge, Some(interviewing(&[Op::Click], false)), &mut fakes).await;
    let kept: Vec<_> = asked
        .steps
        .iter()
        .map(|taken| {
            let reply = taken.reply.as_ref();
            let reply = reply.map(|r| (r.expectation.clone(), r.purpose_guess.clone()));
            (taken.step, reply)
        })
        .collect();
    let said = ("保存される".to_owned(), "経費を申請する".to_owned());
    assert_eq!(kept, [(1, Some(said))], "the step and what the judge said");
}

#[tokio::test]
async fn interview_keeps_the_step_whose_answers_are_unusable() {
    let mut fakes = Fakes::new();
    let judge = Spy::new(false, Scripted::default());
    let asked = asked(judge, Some(interviewing(&[Op::Click], false)), &mut fakes).await;
    let kept: Vec<_> = asked
        .steps
        .iter()
        .map(|t| (t.step, t.reply.is_none()))
        .collect();
    assert_eq!(kept, [(1, true)], "kept, without a reply");
}

#[tokio::test]
async fn other_modes_keep_no_steps() {
    let mut fakes = Fakes::new();
    let asked = asked(Spy::new(false, answers(false)), None, &mut fakes).await;
    assert!(asked.steps.is_empty(), "nothing kept outside the interview");
}
