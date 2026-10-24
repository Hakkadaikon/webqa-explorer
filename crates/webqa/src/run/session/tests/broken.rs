//! Tests of the runs the browser fails in: the records so far are written.

use super::fakes::*;
use super::*;

/// The pages of a run that fails in its second step: the first page, and
/// the look before and after the first click.
const PAGES: usize = 3;

/// Runs the script of [`clicks`] on a browser that shows the top page
/// `pages` times and then fails, writing into `store`.
async fn broken(pages: usize, store: &mut dyn Store) -> Ending {
    let mut user = Counting {
        calls: 0,
        fire_on: BTreeSet::new(),
    };
    run_broken(&mut shown(&top(), pages), &mut user, &plan(), store).await
}

/// A browser that shows `page` `pages` times and then fails.
pub fn shown(page: &Observation, pages: usize) -> FakeBrowser {
    FakeBrowser {
        pages: iter::repeat_n(page.clone(), pages).collect(),
        ..FakeBrowser::default()
    }
}

/// Runs the script of [`clicks`] on `browser` as `plan` says, interrupted
/// by `user`, writing into `store`.
pub async fn run_broken(
    browser: &mut FakeBrowser,
    user: &mut dyn Interrupt,
    plan: &Plan,
    store: &mut dyn Store,
) -> Ending {
    let mut judge = RecordingJudge {
        script: clicks(),
        ..RecordingJudge::default()
    };
    let (mut clock, redactor) = (FakeClock::default(), Redactor::default());
    let mut ports = Ports::new(browser, &mut judge, &mut clock, store, &redactor);
    conduct(&mut ports, user, rules(100), plan).await
}

#[tokio::test]
async fn browser_failure_writes_records_so_far() {
    let mut store = MemoryStore::default();
    let ending = broken(PAGES, &mut store).await;
    assert_eq!(format!("{ending:?}"), "Broke(BrowserError)");
    assert_eq!(exit::code(&ending), 3);
    let last = json!({"kind": "run_finished", "reason": "error", "step": 2});
    assert_eq!(ends(&store).1, Some(last));
    assert!(saved_operated(&store));
    let report = store.report.unwrap();
    let steps = report["steps"].as_array().unwrap().len();
    assert_eq!(
        (&report["reason"], &report["stuck"], steps),
        (&json!("error"), &Value::Null, 1)
    );
}

#[tokio::test]
async fn browser_failure_records_written_in_order() {
    let mut store = Slow::holding(Hold::Once);
    let ending = broken(2, &mut store).await;
    assert_eq!(format!("{ending:?}"), "Broke(BrowserError)");
    let tail = &store.order[store.order.len() - 3..];
    assert_eq!(tail, ["write_memory", "write_event", "write_report"]);
}

#[tokio::test]
async fn browser_failure_before_first_page_writes_no_memory() {
    let mut store = MemoryStore::default();
    let ending = broken(0, &mut store).await;
    assert_eq!(format!("{ending:?}"), "Broke(BrowserError)");
    let last = json!({"kind": "run_finished", "reason": "error"});
    assert_eq!(ends(&store).1, Some(last));
    let report = store.report.unwrap();
    assert_eq!(
        (&report["reason"], &report["steps"], store.memories.len()),
        (&json!("error"), &json!([]), 0)
    );
}

#[tokio::test]
async fn store_failure_after_browser_failure_ends_failed() {
    let mut whole = MemoryStore::default();
    broken(2, &mut whole).await;
    // The lines before the last one: the memory is the next write.
    let mut short = MemoryStore {
        limit: Some(whole.lines.len() - 1),
        ..MemoryStore::default()
    };
    let ending = broken(2, &mut short).await;
    assert_eq!(format!("{ending:?}"), "Failed(Store(StoreError))");
    // The memory failed, so neither the last line nor the report followed.
    assert_eq!(
        (short.report, short.lines.len()),
        (None, whole.lines.len() - 1)
    );
}
