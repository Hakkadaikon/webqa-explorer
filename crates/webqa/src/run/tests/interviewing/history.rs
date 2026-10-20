//! Tests of the recent operations the judge is told in the screen text.

use std::iter;

use super::unmoved::{clicks_then_done, judged, ported};
use super::*;
use crate::run::tests::{Never, page};

/// The screen text of each call of a run of two clicks and a done.
async fn screens() -> Vec<String> {
    let mut fakes = Fakes::new();
    fakes.browser.pages = iter::repeat_n(page("/"), 40).collect();
    let mut judge = Spy::new(false, clicks_then_done());
    let interview = interviewing(&[Op::Click], false);
    let mut ports = ported(&mut fakes, &mut judge).interviewing(interview);
    let rules = scenario(&[Op::Click]);
    let mut never = Never;
    let run = explore(&mut ports, &mut never, rules, Memory::default());
    run.await.unwrap();
    drop(ports);
    judge.screens
}

#[tokio::test]
async fn the_first_step_has_no_recent_operations() {
    let screens = screens().await;
    let first = screens.first().unwrap();
    assert_eq!(
        first.lines().count(),
        2,
        "the two elements and nothing more"
    );
}

#[tokio::test]
async fn later_calls_are_told_the_operations_so_far() {
    let screens = screens().await;
    let screen = "1: button \"保存\"\n2: link \"[[メールアドレス]] に送る\"";
    let section = |lines: &[&str]| format!("{screen}\n\n直近の操作\n{}", lines.join("\n"));
    let click = "1. click 「保存」 → 画面は変わらなかった";
    let link = "[[メールアドレス]] に送る";
    let again = format!("2. click 「{link}」 → 画面は変わらなかった");
    let want = [
        screen.to_owned(),
        screen.to_owned(),
        section(&[click]),
        section(&[click]),
        section(&[click, &again]),
    ];
    assert_eq!(screens, want, "both calls of a step carry the same section");
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
    let mut ports = ported(&mut fakes, &mut judge).interviewing(interviewing(&[Op::Click], false));
    run_to_end(&mut ports).await;
    let from = ports.record().acts.len();
    let goal = "経費を申請する".to_owned();
    let second = Interviewing::new(persona(), goal, with_done(&[Op::Click]), false, (2, from));
    let mut ports = ports.interviewing(second);
    run_to_end(&mut ports).await;
    drop(ports);
    judge.screens
}

#[tokio::test]
async fn an_earlier_trial_is_not_told_to_the_next() {
    let screens = two_trials().await;
    let first_of_second = screens.get(5).unwrap();
    assert_eq!(
        first_of_second.lines().count(),
        2,
        "no section at the start"
    );
}
