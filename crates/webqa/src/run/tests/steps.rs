//! Tests of one step of the run and of the failures that stop it.

use std::collections::VecDeque;

use webqa_core::effect::Touch;
use webqa_core::judge::JudgeError;
use webqa_core::machine::{Anomaly, ExitReason, Note};
use webqa_judge::scripted::Scripted;

use super::super::*;
use super::{Fakes, page, save};

fn line(seq: u64, event: Event) -> Line {
    Line {
        seq,
        elapsed_ms: 1500,
        step: Some(1),
        event,
    }
}

const fn executed(changed: bool) -> Event {
    Event::Executed {
        changed,
        accesses: 1,
    }
}

#[tokio::test]
async fn one_step_asks_once_touches_once_and_logs_the_decision_and_the_result() {
    let mut fakes = Fakes::new();
    let ended = Ok(ExitReason::BudgetExceeded);
    assert_eq!(fakes.reason().await, ended, "no decision left");
    assert_eq!(
        fakes.judge.texts,
        vec!["1: button \"保存\"\n2: link \"[[メールアドレス]] に送る\"".to_owned()],
        "asked once, with the address masked"
    );
    let touch = Touch {
        op: "click".to_owned(),
        target: Some(save()),
        confidence: 0.9,
    };
    assert_eq!(fakes.browser.touches, vec![touch], "touched once");
    let asked = Event::Asked {
        input_tokens: 0,
        output_tokens: 0,
        think_ms: 0,
    };
    let decided = Event::Decided {
        op: "click".to_owned(),
        target: Some("保存".to_owned()),
        p: 0.9,
    };
    let lines = vec![line(0, asked), line(1, decided), line(2, executed(true))];
    assert_eq!(
        fakes.lines(),
        lines,
        "the question, the decision and the result"
    );
}

#[tokio::test]
async fn touching_the_same_page_logs_no_change() {
    let mut fakes = Fakes::new();
    fakes.browser.pages = VecDeque::from([page("/"), page("/"), page("/")]);
    let ended = Ok(ExitReason::BudgetExceeded);
    assert_eq!(fakes.reason().await, ended, "no decision left");
    let unchanged = line(2, executed(false));
    assert_eq!(fakes.lines().get(2), Some(&unchanged), "the page stayed");
}

#[tokio::test]
async fn judge_failure_ends_the_run_without_touching() {
    let mut fakes = Fakes::new();
    fakes.judge.script = Scripted::default();
    let outcome = fakes.explore().await.unwrap();
    let ended = ExitReason::BudgetExceeded;
    assert_eq!(outcome.reason, ended, "no decision left");
    let failed = [Note::Anomaly(Anomaly::Judge(JudgeError::Exhausted))];
    assert_eq!(outcome.state.notes(), failed, "the core records the error");
    assert_eq!(fakes.browser.touches, Vec::new(), "nothing touched");
}

async fn fails_with(breaks: fn(&mut Fakes), error: RunError) {
    let mut fakes = Fakes::new();
    breaks(&mut fakes);
    assert_eq!(fakes.reason().await, Err(error), "the run stops");
}

#[tokio::test]
async fn no_page_to_start_from_stops_the_run() {
    fails_with(|f| f.browser.pages.clear(), BrowserError.into()).await;
}

#[tokio::test]
async fn a_failed_touch_stops_the_run() {
    fails_with(|f| f.browser.broken = true, BrowserError.into()).await;
}

#[tokio::test]
async fn no_page_after_the_touch_stops_the_run() {
    fails_with(|f| drop(f.browser.pages.pop_back()), BrowserError.into()).await;
}

#[tokio::test]
async fn an_unwritable_decision_stops_the_run() {
    fails_with(|f| f.store.limit = Some(1), StoreError.into()).await;
}

#[tokio::test]
async fn an_unwritable_result_stops_the_run() {
    fails_with(|f| f.store.limit = Some(2), StoreError.into()).await;
}
