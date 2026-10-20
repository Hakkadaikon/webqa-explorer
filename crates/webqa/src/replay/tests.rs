//! Tests of the judge that replays a report.

use std::time::Duration;

use serde_json::{Value, json};
use url::Url;
use webqa_core::judge::QuestionId;
use webqa_core::machine::ExitReason;
use webqa_core::redact::{Redacted, Redactor};
use webqa_core::screen::{Screen, ScreenText, Viewport, screen_text};

use super::*;
use crate::record::Call;
use crate::store::report::build;
use crate::store::report::tests::{first_answers, meta, record, report, start_memory};

fn screen() -> Redacted<ScreenText> {
    let screen = Screen {
        url: Url::parse("https://app.example.test/").unwrap(),
        elements: Vec::new(),
    };
    let viewport = Viewport {
        width: 800,
        height: 600,
    };
    screen_text(&Redactor::default().screen(screen), viewport)
}

#[tokio::test]
async fn replay_gives_back_recorded_replies() {
    let (mode, mut replay, _) = Replay::from_report(&report().to_string()).unwrap();
    assert_eq!(mode, Mode::Explore, "the mode of the run");
    let text = screen();
    let replies = [
        replay.ask(&text, &[]).await,
        replay.ask(&text, &[]).await,
        replay.ask(&text, &[]).await,
    ];
    let expected = [
        Ok(first_answers()),
        Err(JudgeError::Timeout),
        Err(JudgeError::Exhausted),
    ];
    assert_eq!(replies, expected, "the replies in order, then none left");
    nan_comes_back_as_nan().await;
}

async fn nan_comes_back_as_nan() {
    let mut record = record();
    let mut answers = first_answers();
    let operation = QuestionId::operation();
    answers
        .table
        .insert(operation.clone(), vec![("back".to_owned(), f64::NAN)]);
    record.calls[0].reply = Ok(answers);
    let report = build(
        &record,
        ExitReason::BudgetExceeded,
        &meta(),
        &start_memory(),
        &Redactor::default(),
    );
    let text = report.into_inner().to_string();
    let (_, mut replay, _) = Replay::from_report(&text).unwrap();
    let back = replay.ask(&screen(), &[]).await.unwrap();
    let p = back.table[&operation][0].1;
    assert!(p.is_nan(), "a missing probability is NaN again: {p}");
}

#[test]
fn unreadable_report_is_error() {
    let mut schema2 = report();
    schema2["schema"] = json!(2);
    let mut calls_text = report();
    calls_text["calls"] = json!("calls");
    let texts = [
        String::new(),
        "{}".to_owned(),
        schema2.to_string(),
        calls_text.to_string(),
    ];
    for text in texts {
        let read = Replay::from_report(&text).map(|(mode, ..)| mode);
        assert_eq!(read, Err(ReplayError), "not a report: {text}");
    }
    assert_eq!(
        ReplayError.to_string(),
        "実行結果を再実行の台本として読めない",
        "a fixed message"
    );
}

#[test]
fn from_report_reads_start_memory() {
    let start = |report: Value| {
        let (_, _, memory) = Replay::from_report(&report.to_string()).unwrap();
        memory
    };
    let mut old = report();
    old.as_object_mut().unwrap().remove("start_memory");
    assert_eq!(
        (start(report()), start(old)),
        (Some(start_memory()), None),
        "an old report has none"
    );
}

#[tokio::test]
async fn replay_of_an_unreadable_answer_stays_unreadable() {
    let unreadable = JudgeError::Unreadable {
        question: "hard_to_see".to_owned(),
    };
    let mut record = record();
    record.calls = vec![Call {
        step: 1,
        reply: Err(unreadable.clone()),
        think: Duration::from_millis(5),
    }];
    let text = build(
        &record,
        ExitReason::BudgetExceeded,
        &meta(),
        &start_memory(),
        &Redactor::default(),
    )
    .into_inner()
    .to_string();
    let (_, mut replay, _) = Replay::from_report(&text).unwrap();
    let screen = screen();
    assert_eq!(replay.ask(&screen, &[]).await, Err(unreadable));
}

#[tokio::test]
async fn replay_of_an_old_malformed_call_stays_malformed() {
    let mut old = report();
    let call = old["calls"][1].as_object_mut().unwrap();
    call.insert("error".to_owned(), json!("malformed"));
    call.remove("question");
    let (_, mut replay, _) = Replay::from_report(&old.to_string()).unwrap();
    let screen = screen();
    replay.ask(&screen, &[]).await.unwrap();
    let second = replay.ask(&screen, &[]).await;
    assert_eq!(second, Err(JudgeError::Malformed));
}
