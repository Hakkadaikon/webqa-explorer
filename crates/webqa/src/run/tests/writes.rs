//! Tests of the run stopping at whichever record cannot be written.

use webqa_core::config::raw::Op;
use webqa_core::judge::QuestionId;
use webqa_core::machine::ExitReason;
use webqa_judge::scripted::Scripted;

use super::super::fakes;
use super::super::*;
use super::events::stage;
use super::timing::crowded;
use super::{Fakes, limited, url};

/// Two buttons named like image files and a console error on every page;
/// a stage done at the start; the judge sees an error and types into a
/// button, then has nothing left to say.
fn busy(fakes: &mut Fakes) {
    let names = [("button", "icon_save.png"), ("button", "icon_edit.png")];
    let mut shown = fakes::page(&url("/"), &names);
    shown.console = vec!["TypeError: x is undefined".to_owned()];
    fakes.stay_on(&shown);
    fakes.judge.script = Scripted::default()
        .then(QuestionId::operation(), [("input", 0.9)])
        .then(QuestionId::target(), [("1", 0.8)])
        .then(QuestionId::error_shown(), [("true", 0.9)])
        .then(QuestionId::goal_reached(), [("true", 0.1)]);
    fakes.rules = limited(&[Op::Click, Op::Input], 2, 10);
    let login = stage("ログイン", Some("/"), None);
    fakes.rules.stages = vec![login, stage("投稿", None, Some("投稿しました"))];
}

/// The window waits for the deadline, as in the timing tests.
fn waiting(fakes: &mut Fakes) {
    fakes.rules = crowded(fakes);
}

/// The fakes set up by `setup`, the store taking `limit` writes.
async fn run_with(
    setup: fn(&mut Fakes),
    limit: Option<usize>,
) -> (Fakes, Result<ExitReason, RunError>) {
    let mut fakes = Fakes::new();
    setup(&mut fakes);
    fakes.store.limit = limit;
    let reason = fakes.reason().await;
    (fakes, reason)
}

/// Fails each write of the run of `setup` in turn: the run stops there.
async fn stops_at_each_write(setup: fn(&mut Fakes), ended: ExitReason) -> Fakes {
    let (fakes, reason) = run_with(setup, None).await;
    assert_eq!(reason, Ok(ended), "the run ends with every write taken");
    for limit in 0..fakes.store.writes {
        let (_, reason) = run_with(setup, Some(limit)).await;
        assert_eq!(reason, Err(StoreError.into()), "stops at write {limit}");
    }
    fakes
}

#[tokio::test]
async fn every_failed_write_stops_the_run() {
    let fakes = stops_at_each_write(busy, ExitReason::BudgetExceeded).await;
    let names: Vec<_> = fakes.store.shots.iter().map(|(n, _)| n.as_str()).collect();
    let shots = [
        "ISSUE_accessible_name_step0.png",
        "ISSUE_console_error_step0.png",
        "ISSUE_error_display_step1.png",
    ];
    assert_eq!(names, shots, "one screenshot for the two names");
    assert_eq!(fakes.record.found.len(), 4, "every issue recorded");
    let kinds = [
        "issue",
        "issue",
        "issue",
        "stage_advanced",
        "asked",
        "issue",
        "decided",
        "skipped",
        "executed",
        "asked",
        "anomaly",
    ];
    assert_eq!(fakes.kinds(), kinds, "every kind of line is written");
    stops_at_each_write(waiting, ExitReason::TimeExceeded).await;
}
