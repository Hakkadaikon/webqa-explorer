//! Tests of the issues recorded when the browser cannot take screenshots.

use std::{future, iter};

use serde_json::Value;
use webqa_core::effect::Touch;
use webqa_core::judge::Image;
use webqa_core::machine::ExitReason;
use webqa_core::ports::{Browser, BrowserError, Observation, Pending};

use super::super::fakes::FakeBrowser;
use super::{Fakes, one, rules, run_on};

/// Cannot take screenshots, otherwise a [`FakeBrowser`].
struct Blind(FakeBrowser);

impl Browser for Blind {
    fn observe(&mut self) -> Pending<'_, Result<Observation, BrowserError>> {
        self.0.observe()
    }

    fn touch<'a>(&'a mut self, touch: &'a Touch) -> Pending<'a, Result<(), BrowserError>> {
        self.0.touch(touch)
    }

    fn screenshot(&mut self) -> Pending<'_, Result<Image, BrowserError>> {
        Box::pin(future::ready(Err(BrowserError)))
    }
}

/// A blind browser on a page whose console has an error.
fn blind() -> Blind {
    let shown = one("保存", &["TypeError: x is undefined"]);
    Blind(FakeBrowser {
        pages: iter::repeat_n(shown, 5).collect(),
        ..FakeBrowser::default()
    })
}

/// The kind of the first line and its screenshot, if it has one.
fn first_line(fakes: &Fakes) -> Option<(Value, Option<Value>)> {
    let first = fakes.store.lines.first();
    first.map(|l| (l["kind"].clone(), l.get("screenshot").cloned()))
}

#[tokio::test]
async fn screenshot_failure_records_issue_without_image() {
    let mut fakes = Fakes::new();
    let mut browser = blind();
    let (judge, clock) = (&mut fakes.judge, &mut fakes.clock);
    let (store, redactor) = (&mut fakes.store, &fakes.redactor);
    let (reason, record) = run_on(&mut browser, judge, clock, store, redactor, rules()).await;
    let shots: Vec<_> = record.found.iter().map(|f| f.shot.clone()).collect();
    assert_eq!(shots, [None], "recorded without a screenshot");
    assert_eq!(reason, Ok(ExitReason::BudgetExceeded), "the run goes on");
    let issue = Some(("issue".into(), None));
    assert_eq!(first_line(&fakes), issue, "no screenshot in the line");
    assert!(fakes.store.shots.is_empty(), "nothing written");
    assert_eq!(fakes.judge.texts.len(), 1, "asked after the issue");
}
