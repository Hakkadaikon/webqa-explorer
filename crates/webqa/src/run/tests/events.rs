//! Tests of the lines the run writes for the effects and the core's notes.

use std::num::NonZeroU32;

use webqa_core::config::raw::{DoneWhen, Op, Stage};
use webqa_core::machine::{Cause, ExitReason, Rejection, StuckKind};

use super::super::*;
use super::{Fakes, limited, page, script, url};

const ONE_STEP: [&str; 3] = ["asked", "decided", "executed"];

#[tokio::test]
async fn store_failure_stops_run() {
    let mut fakes = Fakes::new();
    fakes.store.limit = Some(0);
    let failed = Err(StoreError.into());
    assert_eq!(fakes.reason().await, failed, "the first line fails");
    assert_eq!(fakes.judge.texts.len(), 1, "asked once");
    assert_eq!(fakes.browser.touches, Vec::new(), "nothing touched");
    let mut fakes = Fakes::new();
    fakes.store.limit = Some(2);
    assert_eq!(fakes.reason().await, failed, "the result fails");
    assert_eq!(fakes.kinds(), ONE_STEP, "the result was the third write");
    assert_eq!(fakes.browser.touches.len(), 1, "touched before the failure");
}

#[tokio::test]
async fn events_follow_effects() {
    let mut fakes = Fakes::new();
    let ended = Ok(ExitReason::BudgetExceeded);
    assert_eq!(fakes.reason().await, ended, "one decision");
    assert_eq!(fakes.kinds(), ONE_STEP, "no finish line");
    let numbers: Vec<_> = fakes.lines().iter().map(|l| (l.seq, l.step)).collect();
    let numbered = [(0, Some(1)), (1, Some(1)), (2, Some(1))];
    assert_eq!(numbers, numbered, "one step");
}

#[tokio::test]
async fn core_touches_are_recovered() {
    let mut fakes = Fakes::new();
    fakes.stay_on(&page("/pricing"));
    fakes.judge.script = script(&[]);
    fakes.rules.max_decisions = NonZeroU32::new(20).unwrap();
    let stuck = Ok(ExitReason::Stuck(StuckKind::NoProgress));
    assert_eq!(fakes.reason().await, stuck, "leaving never gets anywhere");
    let expected = [leave(1), leave(2), leave(3)].concat();
    let picked = fakes.picked(&["recovered", "executed"]);
    assert_eq!(picked, expected, "left 3 times");
    assert_eq!(fakes.kinds().len(), 6, "nothing else");
    let acts: Vec<_> = fakes.record.acts.iter().map(|a| (a.cause, a.p)).collect();
    let own = [(Cause::Paywall, 1.0); 3];
    assert_eq!(acts, own, "the core's own operations");
    assert!(fakes.judge.texts.is_empty(), "the judge is not asked");
}

/// The lines of leaving a paywall without effect at `step`.
fn leave(step: u64) -> [(Option<u64>, Event); 2] {
    let left = Event::Recovered {
        op: "back".to_owned(),
        cause: Cause::Paywall,
    };
    let unchanged = Event::Executed {
        changed: false,
        accesses: 1,
    };
    [(Some(step), left), (Some(step), unchanged)]
}

pub(super) fn stage(name: &str, url: Option<&str>, text: Option<&str>) -> Stage {
    Stage {
        name: name.to_owned(),
        goal: format!("{name}する"),
        done_when: DoneWhen {
            url: url.map(str::to_owned),
            element: None,
            text: text.map(str::to_owned),
        },
    }
}

#[tokio::test]
async fn stage_advance_is_logged() {
    let mut fakes = Fakes::new();
    let login = stage("ログイン", Some("/"), None);
    fakes.rules.stages = vec![login, stage("投稿", None, Some("投稿しました"))];
    let _ = fakes.explore().await;
    let advanced = Event::StageAdvanced {
        stage: 1,
        name: "投稿".to_owned(),
    };
    let first = fakes.lines().into_iter().next().map(|l| (l.step, l.event));
    assert_eq!(first, Some((None, advanced)), "before the first step");
}

#[tokio::test]
async fn anomaly_and_rejection_are_logged() {
    let mut fakes = Fakes::new();
    fakes.judge.script = script(&[("back", 0.5, "1"), ("drag", 0.9, "1")]);
    fakes.stay_on(&page("/"));
    fakes.rules.max_decisions = NonZeroU32::new(3).unwrap();
    let ended = Ok(ExitReason::BudgetExceeded);
    assert_eq!(fakes.reason().await, ended, "three questions");
    let anomaly = |cause: &str, question: Option<&str>| Event::Anomaly {
        cause: cause.to_owned(),
        question: question.map(str::to_owned),
        detail: None,
    };
    let why = Rejection::LowConfidence;
    let expected = [
        (Some(1), Event::Rejected { why }),
        (Some(2), anomaly("not_an_option", Some("operation"))),
        (Some(3), anomaly("exhausted", None)),
    ];
    assert_eq!(fakes.picked(&["rejected", "anomaly"]), expected, "in order");
}

/// Explore mode over two fields, "名前" with a value and "備考" without,
/// and a button; `number` is the element the judge types into.
async fn typed_into(number: &str) -> Fakes {
    let mut fakes = Fakes::new();
    fakes.judge.script = script(&[("input", 0.9, number), ("click", 0.9, "1")]);
    let fields = [("textbox", "名前"), ("textbox", "備考"), ("button", "保存")];
    fakes.stay_on(&fakes::page(&url("/"), &fields));
    fakes.rules = limited(&[Op::Click, Op::Input], 2, 10);
    fakes.rules.input_names.insert("名前".to_owned());
    let ended = Ok(ExitReason::BudgetExceeded);
    assert_eq!(fakes.reason().await, ended, "two questions");
    fakes
}

#[tokio::test]
async fn input_that_does_nothing_is_recorded_and_run_continues() {
    let skipped = ["asked", "decided", "skipped", "executed"];
    let kinds = [&skipped[..], &ONE_STEP[..]].concat();
    for (number, cause) in [("3", "not_an_input"), ("2", "no_input_value")] {
        let fakes = typed_into(number).await;
        assert_eq!(
            fakes.kinds(),
            kinds,
            "the input does nothing, the run goes on"
        );
        let skipped = Event::Skipped {
            cause: cause.to_owned(),
        };
        assert_eq!(fakes.picked(&["skipped"]), [(Some(1), skipped)], "{cause}");
    }
}
