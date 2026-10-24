//! Tests of how a run the browser fails in ends: in a replay, and when the
//! user interrupts the writing.

use super::broken::{run_broken, shown};
use super::fakes::*;
use super::*;

/// Interrupts on none of its waits, or on the ones numbered in `fire_on`.
fn user(fire_on: BTreeSet<u32>) -> Counting {
    Counting { calls: 0, fire_on }
}

#[tokio::test]
async fn failure_in_a_replay_writes_no_memory() {
    let replay = Plan {
        start: Start::Given(Memory::default()),
        ..plan()
    };
    let mut store = MemoryStore::default();
    let mut nobody = user(BTreeSet::new());
    let ending = run_broken(&mut shown(&top(), 3), &mut nobody, &replay, &mut store).await;
    assert_eq!(format!("{ending:?}"), "Broke(BrowserError)");
    let reason = store.report.unwrap()["reason"].clone();
    assert_eq!((store.memories.len(), reason), (0, json!("error")));
}

#[tokio::test]
async fn second_interrupt_while_writing_after_failure_abandons() {
    let mut nobody = user(BTreeSet::new());
    let mut whole = MemoryStore::default();
    run_broken(&mut shown(&top(), 3), &mut nobody, &plan(), &mut whole).await;
    // The waits of the run itself: the last one came while writing.
    let waits = nobody.calls - 1;
    let gate = Arc::new(Notify::new());
    let mut store = Slow::holding(Hold::Forever(Arc::clone(&gate)));
    let mut twice = user(BTreeSet::from([waits + 1, waits + 2, waits + 3]));
    let ending = run_broken(&mut shown(&top(), 3), &mut twice, &plan(), &mut store).await;
    assert_eq!(format!("{ending:?}"), "Abandoned");
    assert_eq!(exit::code(&ending), 3);
    // The first interruption while writing is allowed; the second abandons.
    assert_eq!((twice.calls, store.inner.report), (waits + 2, None));
}
