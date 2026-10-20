//! Tests of the screenshot the interview sends with its questions.

use super::*;

#[tokio::test]
async fn interview_sends_screenshot_to_a_judge_that_reads_images() {
    let mut fakes = Fakes::new();
    let judge = Spy::new(true, answers(true));
    let asked = asked(judge, Some(interviewing(&[Op::Click], true)), &mut fakes).await;
    let png = Image::new(ImageFormat::Png, PNG.to_vec());
    let hard = asked.judge.asked.first().unwrap().iter();
    let hard = hard.filter(|q| q.id().as_str() == HARD_TO_SEE).count();
    assert_eq!(
        (asked.judge.images, asked.shots, hard),
        (vec![png], 1, 1),
        "one screenshot, and the question about what is hard to see"
    );
}

#[tokio::test]
async fn interview_keeps_but_does_not_send_a_screenshot_for_a_judge_that_does_not_read_images() {
    let mut fakes = Fakes::new();
    let judge = Spy::new(false, answers(false));
    let asked = asked(judge, Some(interviewing(&[Op::Click], false)), &mut fakes).await;
    assert_eq!(
        (
            asked.judge.images.len(),
            asked.shots,
            asked.steps[0].image.is_some()
        ),
        (0, 1, true),
        "the screenshot was taken and kept, and not sent"
    );
}

#[tokio::test]
async fn interview_asks_without_image_when_the_screenshot_fails() {
    let mut fakes = Fakes::new();
    fakes.browser.unshootable = true;
    let judge = Spy::new(true, answers(true));
    let asked = asked(judge, Some(interviewing(&[Op::Click], true)), &mut fakes).await;
    assert_eq!(
        (
            asked.judge.images.len(),
            asked.judge.asked.len(),
            asked.shots
        ),
        (0, 1, 1),
        "asked as text after the screenshot failed"
    );
}
