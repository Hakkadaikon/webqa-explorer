//! Checks of the judge: the numbering of targets, the table, the time and
//! the wait when the replies run out.

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::sync::Notify;
use webqa_core::judge::{Answers, JudgeError, QuestionId};
use webqa_core::ports::Judge;
use webqa_core::redact::{Redacted, Redactor};
use webqa_core::screen::{ScreenText, screen_text};

use super::{Reply, SceneJudge};
use crate::page::PageSpec;
use crate::scene::poll_once;

/// The screen text of a page of three elements: a button and two links of
/// the same name.
fn screen() -> Redacted<ScreenText> {
    let page = PageSpec::at("/")
        .with(PageSpec::button("保存"))
        .with(PageSpec::link("ヘルプ"))
        .with(PageSpec::link("ヘルプ"))
        .build("https://app.example.test");
    screen_text(&Redactor::default().screen(page.screen), page.viewport)
}

/// What `reply` gives when the judge, thinking `think`, is asked once.
async fn once(reply: Reply, think: Duration) -> (Result<Answers, JudgeError>, u64) {
    let (gate, now) = (Arc::new(Notify::new()), Arc::new(AtomicU64::new(0)));
    let mut judge = SceneJudge::new(VecDeque::from([reply]), gate, Arc::clone(&now), think);
    let answers = judge.ask(&screen(), &[]).await;
    (answers, now.load(Ordering::SeqCst))
}

/// The options of the question `name` in the answers.
fn row(answers: &Result<Answers, JudgeError>, name: &str) -> Vec<(String, f64)> {
    let table = answers
        .iter()
        .filter_map(|a| a.table.get(&QuestionId::new(name)));
    table.flatten().cloned().collect()
}

/// Checks the judge: the numbering of targets, the table, the time and the
/// wait when the replies run out.
pub async fn check() {
    let named = Reply::op("click", 0.9).on("ヘルプ");
    let (answers, _) = once(named, Duration::ZERO).await;
    assert_eq!(
        row(&answers, "target"),
        [("2".to_owned(), 0.9)],
        "the first of that name"
    );
    assert_eq!(
        row(&answers, "operation"),
        [("click".to_owned(), 0.9)],
        "the operation"
    );
    assert_eq!(
        row(&answers, "error_shown"),
        [("true".to_owned(), 0.0)],
        "no error"
    );
    let ranked = Reply::op("click", 0.5)
        .on_nth("ヘルプ", 1)
        .on_number(9)
        .on_ranked("保存", 0.2);
    let (answers, _) = once(ranked, Duration::ZERO).await;
    let targets = [
        ("3".to_owned(), 0.9),
        ("9".to_owned(), 0.9),
        ("1".to_owned(), 0.2),
    ];
    assert_eq!(
        row(&answers, "target"),
        targets,
        "the targets by their numbers"
    );
    check_table().await;
    check_time().await;
    check_stall();
}

/// The other rows of the table, and a reply that fails.
async fn check_table() {
    let (answers, _) = once(Reply::op_without_p("wait"), Duration::ZERO).await;
    assert_eq!(
        row(&answers, "operation")
            .iter()
            .map(|o| o.1.is_nan())
            .collect::<Vec<_>>(),
        [true],
        "no probability"
    );
    let reply = Reply::malformed().error(0.3).goal(0.4).confusing(0.6);
    let (answers, _) = once(reply, Duration::ZERO).await;
    let yes = |name| row(&answers, name);
    assert_eq!(yes("operation"), [], "no operation");
    let rows = [
        yes("error_shown"),
        yes("goal_reached"),
        yes("confusing_display"),
    ];
    let asked = |p: f64| vec![("true".to_owned(), p)];
    assert_eq!(
        rows,
        [asked(0.3), asked(0.4), asked(0.6)],
        "the yes-or-no rows"
    );
    check_failure().await;
}

/// A reply that fails stays a failure, whatever is added to it.
async fn check_failure() {
    let (failed, _) = once(
        Reply::Fail(JudgeError::Api).on("x").at(Duration::ZERO),
        Duration::ZERO,
    )
    .await;
    assert_eq!(failed.err(), Some(JudgeError::Api), "a failure stays one");
}

/// The shared clock moves by the thinking, and up to `at`, never back.
async fn check_time() {
    let late = Reply::op("wait", 0.9).at(Duration::from_secs(10));
    let (_, now) = once(late, Duration::from_secs(5)).await;
    assert_eq!(now, 10_000, "forward to the time of the answer");
    let early = Reply::op("wait", 0.9).at(Duration::from_secs(1));
    let (_, now) = once(early, Duration::from_secs(5)).await;
    assert_eq!(now, 5_000, "not back");
}

/// With no reply left, the judge waits and wakes the gate.
fn check_stall() {
    let (gate, now) = (Arc::new(Notify::new()), Arc::new(AtomicU64::new(0)));
    let mut judge = SceneJudge::new(VecDeque::new(), Arc::clone(&gate), now, Duration::ZERO);
    let screen = screen();
    let mut waiting = judge.ask(&screen, &[]);
    assert!(poll_once(&mut waiting).is_pending(), "it waits");
    let mut woken = Box::pin(gate.notified());
    assert!(poll_once(&mut woken).is_ready(), "the gate knows");
    drop(waiting);
    assert_eq!(
        (judge.stalled, judge.asked, judge.answered),
        (true, 1, 0),
        "it stalled"
    );
}
