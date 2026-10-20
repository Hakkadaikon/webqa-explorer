//! Tests of the answers to the added questions that cannot be used.

use super::*;

/// The answers of a step that clicks the first element, with every added
/// question answered but the elements hard to see, which no image asks for.
fn complete() -> Answers {
    let mut said = Answers::default();
    let mut put = |id: QuestionId, options: Vec<(&str, f64)>| {
        let options = options.into_iter().map(|(o, p)| (o.to_owned(), p));
        said.table.insert(id, options.collect());
    };
    put(QuestionId::operation(), vec![("click", 0.9)]);
    put(QuestionId::target(), vec![("1", 0.8)]);
    put(QuestionId::error_shown(), vec![("true", 0.0)]);
    put(QuestionId::goal_reached(), vec![("true", 0.0)]);
    put(QuestionId::new(HESITATION), vec![("true", 0.2)]);
    put(QuestionId::new(EXPECTATION), vec![("保存される", f64::NAN)]);
    put(
        QuestionId::new(PURPOSE_GUESS),
        vec![("経費を申請する", f64::NAN)],
    );
    said
}

/// Asks with the judge giving `answers`, and says what the interview passed
/// on and kept: the reply, and for each step whether it has no reply.
async fn passed(answers: Answers) -> (Result<Answers, JudgeError>, Vec<(u64, bool)>) {
    let mut fakes = Fakes::new();
    let judge = Spy::giving(answers);
    let asked = asked(judge, Some(interviewing(&[Op::Click], false)), &mut fakes).await;
    let kept = asked.steps.iter().map(|t| (t.step, t.reply.is_none()));
    (asked.reply, kept.collect())
}

#[tokio::test]
async fn interview_passes_on_a_usable_reply_as_it_is() {
    let (reply, kept) = passed(complete()).await;
    // A free sentence has no probability, which is NaN and equals nothing:
    // the answered ids stand for the answers.
    let ids = |answers: Answers| answers.table.into_keys().collect::<Vec<_>>();
    assert_eq!(
        (reply.map(ids), kept),
        (Ok(ids(complete())), vec![(1, false)]),
        "the answers reach the core, and the step keeps its reply"
    );
}

#[tokio::test]
async fn interview_passes_on_a_malformed_reply_when_an_added_answer_is_missing() {
    let mut said = complete();
    said.table.remove(&QuestionId::new(EXPECTATION));
    let (reply, kept) = passed(said).await;
    assert_eq!(
        (reply, kept),
        (Err(JudgeError::Malformed), vec![(1, true)]),
        "the core reads an anomaly, and the step is kept without a reply"
    );
}

#[tokio::test]
async fn interview_passes_on_a_malformed_reply_when_an_added_answer_is_out_of_range() {
    let mut said = complete();
    said.table
        .insert(QuestionId::new(HESITATION), vec![("true".to_owned(), 1.2)]);
    let (reply, kept) = passed(said).await;
    assert_eq!(
        (reply, kept),
        (Err(JudgeError::Malformed), vec![(1, true)]),
        "the core reads an anomaly, and the step is kept without a reply"
    );
}
