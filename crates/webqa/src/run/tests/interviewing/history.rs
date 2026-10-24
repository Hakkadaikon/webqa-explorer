//! Tests of the recent operations the judge is told in the screen text.

use std::collections::VecDeque;
use std::iter;

use webqa_core::ports::Observation;

use super::unmoved::{clicks_then_done, judged, ported};
use super::*;
use crate::run::tests::{Never, page};

/// The screen text of each call of a run of two clicks and a done.
async fn screens() -> Vec<String> {
    let pages = iter::repeat_n(page("/"), 40).collect();
    screens_over(pages).await
}

/// The same run over `pages`, one for each time the browser is read.
async fn screens_over(pages: VecDeque<Observation>) -> Vec<String> {
    let mut fakes = Fakes::new();
    fakes.browser.pages = pages;
    let mut judge = Spy::new(false, clicks_then_done());
    let interview = interviewing(false);
    let mut ports = ported(&mut fakes, &mut judge).interviewing(interview);
    let rules = scenario(&[Op::Click]);
    let mut never = Never;
    let run = explore(&mut ports, &mut never, rules, Memory::default());
    run.await.unwrap();
    drop(ports);
    judge.screens
}

/// The elements of the page, as the judge is shown them.
const ELEMENTS: &str = "1: button \"保存\"\n2: link \"[[メールアドレス]] に送る\"";

/// The screen text of each call of a trial of two clicks and a done on a
/// page that does not change.
fn trial_screens() -> Vec<String> {
    let section = |lines: &[&str]| format!("{ELEMENTS}\n\n直近の操作\n{}", lines.join("\n"));
    let click = "1. click 「保存」 → 画面は変わらなかった";
    let again = "2. click 「[[メールアドレス]] に送る」 → 画面は変わらなかった";
    vec![
        ELEMENTS.to_owned(),
        ELEMENTS.to_owned(),
        section(&[click]),
        section(&[click]),
        section(&[click, again]),
    ]
}

#[tokio::test]
async fn later_calls_are_told_the_operations_so_far() {
    let screens = screens().await;
    assert_eq!(
        screens,
        trial_screens(),
        "both calls of a step carry the same section"
    );
}

/// Runs the core on `ports` to its end.
async fn run_to_end(ports: &mut Ports<'_>) {
    let mut never = Never;
    let run = explore(ports, &mut never, scenario(&[Op::Click]), Memory::default());
    run.await.unwrap();
}

/// The screen text of each call of two runs of two clicks and a done, the
/// second told to start its history where the first ended.
async fn two_trials() -> Vec<String> {
    let mut fakes = Fakes::new();
    fakes.browser.pages = iter::repeat_n(page("/"), 80).collect();
    let script = judged(&["click", "click", "done", "click", "click", "done"]);
    let mut judge = Spy::new(false, script);
    let mut ports = ported(&mut fakes, &mut judge).interviewing(interviewing(false));
    run_to_end(&mut ports).await;
    let from = ports.record().acts.len();
    let goal = "経費を申請する".to_owned();
    let second = Interviewing::new(persona(), goal, false, 2, from);
    let mut ports = ports.interviewing(second);
    run_to_end(&mut ports).await;
    drop(ports);
    judge.screens
}

#[tokio::test]
async fn an_earlier_trial_is_not_told_to_the_next() {
    let screens = two_trials().await;
    let second = screens.get(5..).unwrap();
    assert_eq!(
        second,
        trial_screens(),
        "numbered from 1, its own operations"
    );
}
#[tokio::test]
async fn a_page_that_changed_is_told_as_changed() {
    let before = iter::repeat_n(page("/"), 3);
    let after = iter::repeat_n(page("/next"), 40);
    let screens = screens_over(before.chain(after).collect()).await;
    let last = screens.last().unwrap();
    let want = format!(
        "{ELEMENTS}\n\n直近の操作\n{}\n{}",
        "1. click 「保存」 → 画面は変わらなかった",
        "2. click 「[[メールアドレス]] に送る」 → 画面が変わった"
    );
    assert_eq!(last, &want, "the second click moved the page to /next");
}
