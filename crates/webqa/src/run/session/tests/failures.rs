//! Tests of the runs that go wrong.

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
    let mut short = MemoryStore {
        limit: Some(before_memory),
        ..MemoryStore::default()
    };
    let ending = on_top(Wait::Step, &mut short).await;
    assert_eq!(format!("{ending:?}"), "Failed(Store(StoreError))");
    assert_eq!(short.report, None);
    assert_eq!(short.lines.len(), before_memory);
}

#[tokio::test]
async fn confirmed_issue_sets_ending() {
    let url = Url::parse("https://app.example.test/").unwrap();
    let page = base::page(&url, &[("button", "icon_save.png")]);
    let mut store = MemoryStore::default();
    let gate = Arc::new(Notify::new());
    let ending = interrupted(Wait::Question(0), &page, &gate, &mut store).await;
    assert_eq!(exit::code(&ending), 1);
    assert_eq!(store.report.unwrap()["issues"].as_array().unwrap().len(), 1);
}
