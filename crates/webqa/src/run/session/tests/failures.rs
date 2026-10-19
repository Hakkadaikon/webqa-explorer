//! Tests of the runs that go wrong.

use super::fakes::*;
use super::*;

#[tokio::test]
async fn broken_memory_is_logged() {
    let mut store = MemoryStore::default();
    store.memories.insert(ORIGIN.to_owned(), "{".to_owned());
    let ending = on_top(Wait::Step, &mut store).await;
    assert_eq!(exit::code(&ending), 0);
    assert_eq!(store.aside, [(ORIGIN.to_owned(), "{".to_owned())]);
    let reset = json!({"kind": "memory_reset", "cause": "unreadable"});
    assert_eq!(kinds(&store).get(1), Some(&reset));
    let written = Memory::from_json(&store.memories[ORIGIN], ORIGIN).unwrap();
    assert_eq!(written.screen_count(), 1);
}

#[tokio::test]
async fn store_failure_ends_failed() {
    let mut full = MemoryStore {
        limit: Some(0),
        ..MemoryStore::default()
    };
    let ending = on_top(Wait::Step, &mut full).await;
    assert_eq!(format!("{ending:?}"), "Failed(Store(StoreError))");
    assert_eq!(exit::code(&ending), 3);
    assert_eq!((full.report, full.memories.len()), (None, 0));
    let mut whole = MemoryStore::default();
    on_top(Wait::Step, &mut whole).await;
    // The lines before the last one: the memory is the next write.
    let before_memory = whole.lines.len() - 1;
    for (extra, lines, report_written) in [(0, 0, false), (1, 1, false), (2, 1, true)] {
        let (ending, short) = fails_after(before_memory + extra).await;
        assert_eq!(format!("{ending:?}"), "Failed(Store(StoreError))");
        assert_eq!(exit::code(&ending), 3);
        assert_eq!(short.lines.len(), before_memory + lines);
        assert_eq!(short.report.is_some(), report_written);
    }
}

/// The run of the top page, interrupted at a step, on a store that takes
/// `limit` writes.
async fn fails_after(limit: usize) -> (Ending, MemoryStore) {
    let mut store = MemoryStore {
        limit: Some(limit),
        ..MemoryStore::default()
    };
    let ending = on_top(Wait::Step, &mut store).await;
    (ending, store)
}

#[tokio::test]
async fn a_log_that_fails_during_the_run_ends_failed() {
    let (ending, short) = fails_after(2).await;
    assert_eq!(format!("{ending:?}"), "Failed(Store(StoreError))");
    assert_eq!(exit::code(&ending), 3);
    assert_eq!((short.report, short.memories.len()), (None, 0));
}

#[tokio::test]
async fn confirmed_issue_sets_ending() {
    let url = Url::parse("https://app.example.test/").unwrap();
    let page = base::page(&url, &[("button", "icon_save.png")]);
    let mut store = MemoryStore::default();
    let gate = Arc::new(Notify::new());
    let ending = interrupted(Wait::Question(0), &page, &gate, &mut store).await;
    assert_eq!(exit::code(&ending), 1);
    let issues = store.report.unwrap()["issues"].clone();
    assert_eq!(issues.as_array().unwrap().len(), 1);
    assert_eq!(issues[0]["kind"], "accessible_name");
}

#[tokio::test]
async fn unreadable_memory_ends_failed() {
    let mut store = Slow::holding(Hold::Once);
    store.readable = false;
    let ending = on_top(Wait::Step, &mut store).await;
    assert_eq!(format!("{ending:?}"), "Failed(Store(StoreError))");
    assert_eq!(exit::code(&ending), 3);
    assert_eq!(store.inner.report, None);
}

#[tokio::test]
async fn a_reset_that_cannot_be_logged_ends_failed() {
    let mut store = MemoryStore {
        limit: Some(1),
        ..MemoryStore::default()
    };
    store.memories.insert(ORIGIN.to_owned(), "{".to_owned());
    let ending = on_top(Wait::Step, &mut store).await;
    assert_eq!(format!("{ending:?}"), "Failed(Store(StoreError))");
    assert_eq!(exit::code(&ending), 3);
    assert_eq!((store.report, store.memories.len()), (None, 0));
}
