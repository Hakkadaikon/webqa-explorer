//! Tests of the ports that go on from what an earlier browser's ports kept.

use std::time::Duration;

use webqa_core::judge::JudgeError;

use super::super::*;
use super::fakes::{FakeBrowser, FakeClock, MemoryStore, RecordingJudge};
use crate::record::Call;

/// The ports of one browser after another, over one store.
#[derive(Default)]
struct Parts {
    browser: FakeBrowser,
    judge: RecordingJudge,
    clock: FakeClock,
    store: MemoryStore,
    redactor: Redactor,
}

impl Parts {
    fn ports(&mut self) -> Ports<'_> {
        let (browser, judge) = (&mut self.browser, &mut self.judge);
        Ports::new(
            browser,
            judge,
            &mut self.clock,
            &mut self.store,
            &self.redactor,
        )
    }
}

const WAITED: Event = Event::Waited { until_ms: 1 };

#[tokio::test]
async fn resumed_ports_go_on_from_what_was_kept() {
    let mut parts = Parts::default();
    let mut first = parts.ports();
    first.log(WAITED).await.unwrap();
    first.record.calls.push(Call {
        step: 1,
        reply: Err(JudgeError::Exhausted),
        think: Duration::ZERO,
    });
    let kept = first.into_kept();
    let mut second = parts.ports().resumed(kept);
    second.log(WAITED).await.unwrap();
    let calls = second.record().calls.len();
    let lines = &parts.store.lines;
    let seqs: Vec<_> = lines.iter().filter_map(|l| l["seq"].as_u64()).collect();
    assert_eq!((seqs, calls), (vec![0, 1], 1), "the numbers and the record");
}

#[tokio::test]
async fn failed_write_is_kept_for_the_next_ports() {
    let mut parts = Parts::default();
    parts.store.limit = Some(0);
    let mut first = parts.ports();
    let failed = first.log(WAITED).await;
    let kept = first.into_kept();
    let again = parts.ports().resumed(kept).log(WAITED).await;
    let error = Err(StoreError.into());
    assert_eq!(
        (failed, again, parts.store.writes),
        (error, error, 1),
        "the second ports write nothing"
    );
}

#[tokio::test]
async fn resumed_ports_keep_the_names_of_the_screenshots_written() {
    let mut parts = Parts::default();
    let mut first = parts.ports();
    first.shots.insert("ISSUE_a.png".to_owned());
    let kept = first.into_kept();
    let second = parts.ports().resumed(kept);
    let names: Vec<_> = second.shots.iter().map(String::as_str).collect();
    assert_eq!(
        names,
        ["ISSUE_a.png"],
        "the names of the first browser's shots"
    );
}
