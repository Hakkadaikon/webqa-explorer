//! Checks of the judge: the replies in order, what it writes down, and the
//! wait when the replies run out.

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::sync::Notify;
use webqa_core::judge::{Answers, Image, ImageFormat, JudgeError, Question, QuestionId};
use webqa_core::ports::Judge;
use webqa_core::redact::{Redacted, Redactor};
use webqa_core::screen::{ScreenText, screen_text};

use super::ORIGIN;
use crate::judge::{Asked, Kind as Asks, Said, SceneJudge};
use crate::page::PageSpec;
use crate::scene::poll_once;

/// The three kinds of questions, one each.
fn questions() -> Vec<Question> {
    vec![
        Question::Noul {
            id: QuestionId::new("a"),
            prompt: "真偽".to_owned(),
        },
        Question::Choice {
            id: QuestionId::new("b"),
            prompt: "選択".to_owned(),
            options: vec!["1".to_owned(), "2".to_owned()],
        },
        Question::Text {
            id: QuestionId::new("c"),
            prompt: "自由".to_owned(),
        },
    ]
}

/// A judge with the replies `said`, and the clock it shares.
fn judge(said: Vec<Said>) -> (SceneJudge, Arc<Notify>, Arc<AtomicU64>) {
    let (gate, now) = (Arc::new(Notify::new()), Arc::new(AtomicU64::new(0)));
    let replies: VecDeque<_> = said.into_iter().map(|said| Ok(said.build())).collect();
    let judge = SceneJudge::new(
        replies,
        Arc::clone(&gate),
        Arc::clone(&now),
        Duration::from_secs(2),
    );
    (judge, gate, now)
}

/// The screen text of a page with one button.
fn screen() -> Redacted<ScreenText> {
    let page = PageSpec::at("/").with(PageSpec::button("保存"));
    let page = page.build(ORIGIN);
    screen_text(&Redactor::default().screen(page.screen), page.viewport)
}

/// The reply as the best option of the question `id`.
fn best(reply: &Result<Answers, JudgeError>, id: &str) -> Option<String> {
    let answers = reply.as_ref().ok()?;
    let (option, _) = answers.best(&QuestionId::new(id))?;
    Some(option.to_owned())
}

/// The judge gives the replies in order, and takes the time it thinks.
async fn check_replies() {
    let first = Said::default().choose("a", "true", 0.9);
    let second = Said::default().write("c", "文");
    let (mut judge, _, now) = judge(vec![first, second]);
    let asked = questions();
    let one = judge.ask(&screen(), &asked).await;
    let two = judge.ask(&screen(), &asked).await;
    assert_eq!(best(&one, "a"), Some("true".to_owned()), "the first reply");
    assert_eq!(best(&two, "c"), None, "a sentence has no probability");
    let text = two
        .ok()
        .and_then(|a| a.text(&QuestionId::new("c")).map(str::to_owned));
    assert_eq!(text, Some("文".to_owned()), "the second reply");
    assert_eq!(now.load(Ordering::SeqCst), 4000, "two calls took 2 s each");
}

/// The judge writes down the kinds of the questions, the image and the
/// screen.
async fn check_records() {
    let (mut judge, _, _) = judge(vec![Said::default(), Said::default()]);
    let asked = questions();
    judge.ask(&screen(), &asked).await.ok();
    let image = Image::new(ImageFormat::Png, vec![1]);
    let last = asked.get(2..).unwrap_or_default();
    judge.ask_with_image(&screen(), &image, last).await.ok();
    let kinds: Vec<_> = judge.asked.iter().map(Asked::kinds).collect();
    let expected = [
        vec![Asks::Truth, Asks::Choice, Asks::Text],
        vec![Asks::Text],
    ];
    assert_eq!(kinds, expected, "the kinds of the questions");
    check_marks(&judge.asked);
}

/// Which calls had an image, and what screen they sent.
fn check_marks(asked: &[Asked]) {
    let images: Vec<bool> = asked.iter().map(|asked| asked.image).collect();
    assert_eq!(images, [false, true], "which calls had an image");
    let shown = asked.iter().map(|a| a.screen.contains("保存"));
    assert_eq!(shown.collect::<Vec<_>>(), [true, true], "the screen text");
}

/// Out of replies, the judge wakes the gate and waits instead of failing
/// or ending: a run that asks too much is ended by the interrupt.
fn check_exhausted() {
    let (mut judge, gate, _) = judge(vec![]);
    let asked = questions();
    let shown = screen();
    let mut waiting = judge.ask(&shown, &asked);
    assert!(poll_once(&mut waiting).is_pending(), "it waits");
    drop(waiting);
    let mut woken = Box::pin(gate.notified());
    assert!(poll_once(&mut woken).is_ready(), "it woke the gate");
    assert!(judge.stalled, "it says it stalled");
}

/// Checks the judge.
pub async fn check() {
    check_replies().await;
    check_records().await;
    check_exhausted();
}
