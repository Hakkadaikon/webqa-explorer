//! Tests of sending the questions.

use super::*;

#[test]
fn attaches_screenshot_when_judge_reads_images() {
    let shot = shot();
    let mut judge = Recorder::new(true);
    let image = attached(judge.reads_images(), Some(&shot));
    assert_eq!(image, Some(&shot), "the judge reads images");
    let qs = questions(&step(image));
    let text = cart_text();
    sent(&mut judge, &text, image, &qs);
    let calls: Vec<_> = judge
        .calls
        .into_iter()
        .map(|call| (call.screen, call.image))
        .collect();
    let bytes = b"shot of test@example.test".to_vec();
    let want = vec![(text.as_inner().as_str().to_owned(), Some(bytes))];
    assert_eq!(calls, want, "one call with the screen text and the image");
}

const SIX_IDS: &[&str] = &[
    "operation",
    "target",
    "error_shown",
    "goal_reached",
    "hesitation",
    "purpose_guess",
];
const SEVEN_IDS: &[&str] = &[
    "operation",
    "target",
    "error_shown",
    "goal_reached",
    "hesitation",
    "hard_to_see",
    "purpose_guess",
];

/// Whether the judge reads images and whether a screenshot was taken, with
/// whether it is attached and the questions asked.
const CAPABILITY: [(bool, bool, bool, &[&str]); 4] = [
    (true, true, true, SEVEN_IDS),
    (false, true, false, SIX_IDS),
    (true, false, false, SIX_IDS),
    (false, false, false, SIX_IDS),
];

#[test]
fn questions_follow_declared_capability() {
    let shot = shot();
    for (reads, taken, sent, want) in CAPABILITY {
        let image = attached(reads, taken.then_some(&shot));
        assert_eq!(image, sent.then_some(&shot), "{reads} {taken}");
        let qs = questions(&step(image));
        assert_eq!(ids(&qs), want, "{reads} {taken}");
    }
}

#[test]
fn no_screenshot_when_judge_cannot_read_images() {
    let shot = shot();
    let mut judge = Recorder::new(false);
    let image = attached(judge.reads_images(), Some(&shot));
    assert_eq!(image, None, "the judge reads no images");
    let qs = questions(&step(image));
    let text = cart_text();
    sent(&mut judge, &text, image, &qs);
    sent(&mut judge, &text, Some(&shot), &qs);
    let calls: Vec<_> = judge
        .calls
        .iter()
        .map(|call| (call.image.clone(), ids(&call.questions)))
        .collect();
    let bytes = b"shot of test@example.test".to_vec();
    let want = vec![(None, SIX_IDS.to_vec()), (Some(bytes), SIX_IDS.to_vec())];
    assert_eq!(calls, want, "ask without an image, then ask_with_image");
}

#[test]
fn screenshot_sent_unredacted_text_redacted() {
    let shot = shot();
    let mut judge = Recorder::new(true);
    let image = attached(true, Some(&shot));
    sent(&mut judge, &secret_text(), image, &questions(&step(image)));
    let calls: Vec<_> = judge
        .calls
        .into_iter()
        .map(|call| (call.screen, call.image))
        .collect();
    let bytes = b"shot of test@example.test".to_vec();
    let want = vec![("1: text \"[[メールアドレス]]\"".to_owned(), Some(bytes))];
    assert_eq!(calls, want, "the text redacted, the image as taken");
}

#[test]
fn screen_text_redacted_in_interview() {
    let mut judge = Recorder::new(false);
    sent(&mut judge, &secret_text(), None, &questions(&step(None)));
    let leaks: Vec<_> = judge
        .calls
        .iter()
        .map(|call| {
            let prompts = call.questions.iter().map(prompt);
            (
                call.screen.contains("test@example.test"),
                call.screen.contains("[[メールアドレス]]"),
                prompts.filter(|p| p.contains("test@example.test")).count(),
            )
        })
        .collect();
    assert_eq!(leaks, vec![(false, true, 0)], "the secret appears nowhere");
}
