//! Tests of the issues the run finds in the answers, the pages and the
//! core's notes.

use url::Url;
use webqa_core::config::raw::Op;
use webqa_core::issue::{Issue, IssueKind};
use webqa_core::judge::QuestionId;
use webqa_judge::scripted::Scripted;

use super::super::fakes;
use super::{Fakes, limited, one, page, url};
use crate::record::Found;

/// What is compared of each issue found: its kind, evidence, step and
/// screenshot.
type Noted = (IssueKind, Option<String>, u64, Option<String>);

/// Scrolls down `n` times, the judge seeing an error at `error` each time.
fn scrolling(fakes: &mut Fakes, n: u32, error: f64) {
    let answer = |s: Scripted| {
        s.then(QuestionId::operation(), [("scroll_down", 0.9)])
            .then(QuestionId::target(), [("1", 0.8)])
            .then(QuestionId::error_shown(), [("true", error)])
    };
    fakes.judge.script = (0..n).fold(Scripted::default(), |s, _| answer(s));
    fakes.rules = limited(&[Op::ScrollDown], n, 10);
}

fn found(fakes: &Fakes) -> Vec<Noted> {
    let found = fakes.record.found.iter();
    let evidence = |i: &Issue| i.evidence.as_ref().map(|e| e.as_inner().clone());
    let seen = |f: &Found| (f.issue.kind, evidence(&f.issue), f.step, f.shot.clone());
    found.map(seen).collect()
}

fn issue(kind: IssueKind, evidence: Option<&str>, step: u64, shot: &str) -> Noted {
    let evidence = evidence.map(str::to_owned);
    (kind, evidence, step, Some(shot.to_owned()))
}

#[tokio::test]
async fn error_display_saves_screenshot_once() {
    let mut fakes = Fakes::new();
    scrolling(&mut fakes, 2, 0.9);
    fakes.stay_on(&page("/"));
    let _ = fakes.explore().await.unwrap();
    let shot = "ISSUE_error_display_step1.png";
    let error = issue(IssueKind::ErrorDisplay, None, 1, shot);
    assert_eq!(found(&fakes), [error], "recorded once");
    let p = fakes.record.found.first().and_then(|f| f.issue.p);
    assert_eq!(p, Some(0.9), "with the judge's probability");
    let names: Vec<_> = fakes.store.shots.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, [shot], "one screenshot written");
    assert_eq!(fakes.browser.shots, 1, "one screenshot taken");
    let issues = fakes.kinds().into_iter().filter(|k| k == "issue").count();
    assert_eq!(issues, 1, "one issue line");
}

#[tokio::test]
async fn observation_issues_are_recorded() {
    let mut fakes = Fakes::new();
    scrolling(&mut fakes, 1, 0.0);
    let console = "TypeError: x is undefined";
    fakes.stay_on(&one("icon_save.png", &[console]));
    let _ = fakes.explore().await.unwrap();
    let shot = "ISSUE_accessible_name_step0.png";
    let name = issue(IssueKind::AccessibleName, Some("icon_save.png"), 0, shot);
    let shot = "ISSUE_console_error_step0.png";
    let error = issue(IssueKind::ConsoleError, Some(console), 0, shot);
    assert_eq!(found(&fakes), [name, error], "once each, at the start");
}

/// The link on the start page leads to `other`; two decisions.
fn going_out(fakes: &mut Fakes, other: &Url) {
    let start = fakes::page(&url("/"), &[("link", "外部の記事")]);
    let outside = fakes::page(other, &[("link", "戻る")]);
    let pages = [start.clone(), start.clone(), outside, start.clone(), start];
    fakes.browser.pages = pages.into_iter().collect();
    fakes.rules = limited(&[Op::Click, Op::Back], 2, 10);
}

#[tokio::test]
async fn outside_issue_is_recorded() {
    let mut fakes = Fakes::new();
    let other = Url::parse("https://other.example.net/").unwrap();
    going_out(&mut fakes, &other);
    let _ = fakes.explore().await.unwrap();
    let shot = "ISSUE_outside_domain_step1.png";
    let left = issue(IssueKind::OutsideDomain, Some(other.as_str()), 1, shot);
    assert_eq!(found(&fakes), [left], "the page outside");
    let kinds = fakes.kinds();
    let at = |kind: &str| kinds.iter().position(|k| k == kind);
    assert_eq!(at("issue"), Some(3), "after the result");
    assert_eq!(at("recovered"), Some(4), "the return after the screenshot");
}
